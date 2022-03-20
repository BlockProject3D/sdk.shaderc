// Copyright (c) 2022, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use bp3d_threads::{ScopedThreadManager, ThreadPool, UnscopedThreadManager};
use bpx::shader::ShaderPack;
use bpx::shader::symbol::{FLAG_EXTERNAL, FLAG_INTERNAL, Type};
use log::{debug, error, info};
use sha2::Sha512;
use bp3d_symbols::{ConstantObject, OutputObject, PipelineObject, StructObject, TextureObject};
use bp3d_symbols::FromBpx;
use sha2::Digest;
use crate::ext_data::IntoExtData;
use crate::tree::{Symbol, SymbolTree};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("BPX error: {0}")]
    Bpx(bpx::shader::error::Error),
    #[error("BPX deserialization error: {0}")]
    Serde(bpx::sd::serde::Error)
}

bpx::impl_err_conversion!(
    LoadError {
        std::io::Error => Io,
        bpx::shader::error::Error => Bpx,
        bpx::sd::serde::Error => Serde
    }
);

#[derive(Debug, Error)]
pub enum SigningError {
    #[error("binary translation error: {0}")]
    Serde(bincode::Error),

    // A reference has been broken.
    #[error("broken symbol reference")]
    BrokenReference,

    // This means 2 symbols have the same name but different signatures which means they were
    // defined differently...
    #[error("multiple definitions of symbol")]
    SignatureMismatch
}

bpx::impl_err_conversion!(
    SigningError {
        bincode::Error => Serde
    }
);

#[derive(Debug, Error)]
pub enum Error {
    #[error("load error: {0}")]
    Load(LoadError),
    #[error("signing error: {0}")]
    Signing(SigningError)
}

fn load_symbols_single(shader: &Path) -> Result<Vec<Symbol>, LoadError>
{
    debug!("Loading symbols for shader pack {:?}...", shader);
    let mut syms = Vec::new();
    let file = BufReader::new(File::open(shader)?);
    let shaderpack = ShaderPack::open(file)?;
    let symbols = shaderpack.symbols()?;
    for (index, info) in symbols.iter().enumerate() {
        let should_skip = match shaderpack.get_settings().ty {
            //should we be a pipeline and we got an internal symbol, skip!
            bpx::shader::Type::Pipeline => info.flags & FLAG_INTERNAL != 0,
            //otherwise if we're just an assembly, we load all symbols, no matter if they are
            // internal or external.
            bpx::shader::Type::Assembly => false
        };
        if should_skip {
            debug!("Skipping symbol index '{}' ({:?})", index, info);
            continue;
        }
        let name = symbols.load_name(info)?.into();
        let val = symbols.load_extended_data(info)?;
        let ext_data;
        if !val.is_null() {
            ext_data = match info.ty {
                Type::Texture => Some(TextureObject::from_bpx(val)?.into_ext_data()),
                Type::Sampler => None,
                Type::ConstantBuffer => Some(StructObject::from_bpx(val)?.into_ext_data()),
                Type::Constant => Some(ConstantObject::from_bpx(val)?.into_ext_data()),
                Type::VertexFormat => Some(StructObject::from_bpx(val)?.into_ext_data()),
                Type::Pipeline => Some(PipelineObject::from_bpx(val)?.into_ext_data()),
                Type::Output => Some(OutputObject::from_bpx(val)?.into_ext_data())
            };
        } else {
            ext_data = None;
        }
        debug!("Loaded symbol '{}' with index {}", name, index);
        syms.push(Symbol::new(name, index, *info, ext_data));
    }
    Ok(syms)
}

pub fn load_symbols<'a>(n_threads: usize, shaders: impl Iterator<Item = &'a Path>) -> Result<Vec<Symbol>, LoadError> {
    crossbeam::scope(|scope| {
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<Vec<Symbol>, LoadError>> = ThreadPool::new(n_threads);
        info!("Initialized thread pool with {} max thread(s)", n_threads);
        for shader in shaders {
            pool.send(&manager, move |_| load_symbols_single(shader));
            debug!("Dispatch shader pack {:?}", shader);
        }
        pool.reduce().to_vec().unwrap()
    }).unwrap()
}

// This pass pre-hashes symbols to provide reference agnostic signatures
// (they must be paired with signatures of referenced symbols to be complete)
fn pre_hash(n_threads: usize, syms: Vec<Symbol>) -> Result<Vec<Symbol>, SigningError> {
    let manager = UnscopedThreadManager::new();
    let mut pool: ThreadPool<UnscopedThreadManager, Result<Symbol, SigningError>> = ThreadPool::new(n_threads);
    info!("Initialized thread pool with {} max thread(s)", n_threads);
    for mut sym in syms {
        debug!("Dispatch symbol {}", sym.name());
        pool.send(&manager, |_| {
            let (a, b) = sym.get_coded_info();
            let mut v: Vec<u8> = vec![a, b];
            let refs = sym.ext_data().map(|v| v.refs()).unwrap_or(&[]);
            if let Some(ext_data) = sym.ext_data() {
                if refs.len() > 0 {
                    v.extend(ext_data.clone_erase_refs().to_binary()?);
                } else {
                    v.extend(ext_data.to_binary()?);
                }
            }
            //Use SHA 512 to reduce chances of collisions.
            let mut sha = Sha512::new();
            sha.update(v);
            let hash = sha.finalize();
            //let array: [u8; 64] = hash.into();
            sym.set_signature(hash.into());
            Ok(sym)
        });
    }
    pool.reduce().map(|v| v.unwrap()).collect()
}

fn sign_symbols(n_threads: usize, symbols: Vec<Symbol>) -> Result<SymbolTree, SigningError>
{
    let mut tree = SymbolTree::empty();
    let mut hashed = pre_hash(n_threads, symbols)?;
    let items: Result<Vec<(usize, [u8; 64])>, SigningError> = crossbeam::scope(|scope| {
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<(usize, [u8; 64]), SigningError>> = ThreadPool::new(n_threads);
        info!("Initialized thread pool with {} max thread(s)", n_threads);
        for sym in &hashed {
            pool.send(&manager, |index| {
                let refs = sym.ext_data().map(|v| v.refs()).unwrap_or(&[]);
                let sig = sym.signature().unwrap();
                if refs.is_empty() {
                    return Ok((index, *sig))
                }
                let mut allocated = vec![0 as u8; (refs.len() + 1) * 64].into_boxed_slice();
                allocated[0..64].copy_from_slice(&sig[0..64]);
                for (index, r) in refs.iter().enumerate() {
                    let sym1 = match hashed.iter().find(|v| v.index() == *r) {
                        None => {
                            error!("Broken symbol reference '{}'", r);
                            return Err(SigningError::BrokenReference);
                        },
                        Some(v) => v
                    };
                    let sig = sym1.signature().unwrap();
                    allocated[((index + 1) * 64)..].copy_from_slice(&sig[0..64]);
                }
                let mut new_sig = Sha512::new();
                new_sig.update(&allocated);
                let hash = new_sig.finalize();
                Ok((index, hash.into()))
            });
            debug!("Dispatch symbol {}", sym.name());
        }
        pool.reduce().map(|v| v.unwrap()).collect()
    }).unwrap();
    for (index, new_sig) in items? {
        hashed[index].set_signature(new_sig);
    }
    for new in hashed {
        if let Some(existing) = tree.get_by_name(new.name()) {
            if existing.signature().unwrap() != new.signature().unwrap() {
                error!("Duplicate definition of symbol '{}' (first signature: {:X?}, second signature: {:X?})", existing.name(), existing.signature().unwrap(), new.signature().unwrap());
                return Err(SigningError::SignatureMismatch)
            }
        } else {
            tree.insert(new);
        }
    }
    Ok(tree)
}

pub fn load_and_sign_symbols<'a>(n_threads: usize, shaders: impl Iterator<Item = &'a Path>) -> Result<SymbolTree, Error> {
    let syms = load_symbols(n_threads, shaders).map_err(Error::Load)?;
    sign_symbols(n_threads, syms).map_err(Error::Signing)
}

pub fn check_signature_with_assembly(tree: &mut SymbolTree, assembly: &SymbolTree) -> Result<(), SigningError> {
    tree.iter_mut().try_for_each(|new| {
        if let Some(existing) = assembly.get_by_name(new.name()) {
            //The symbol is already part of the parent assembly. Check signature and mark it as
            // EXTERNAL.
            if existing.signature().unwrap() != new.signature().unwrap() {
                error!("Duplicate definition of symbol '{}' (first signature: {:X?}, second signature: {:X?})", existing.name(), existing.signature().unwrap(), new.signature().unwrap());
                return Err(SigningError::SignatureMismatch)
            }
            new.info_mut().flags &= !FLAG_INTERNAL;
            new.info_mut().flags |= FLAG_EXTERNAL;
        }
        Ok(())
    })
}

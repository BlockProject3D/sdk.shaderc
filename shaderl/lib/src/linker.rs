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
use bp3d_threads::{ScopedThreadManager, ThreadPool};
use bpx::shader::ShaderPack;
use bpx::shader::symbol::{FLAG_ASSEMBLY, FLAG_EXTERNAL};
use log::{debug, error, info, warn};

use thiserror::Error;
use crate::symbols::{check_signature_with_assembly, load_and_sign_symbols};

pub struct Config<'a> {
    pub n_threads: usize,
    pub assembly: &'a Path,
    pub shaders: Vec<&'a Path>
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("symbol error: {0}")]
    Symbol(crate::symbols::Error),
    #[error("unresolved external symbol")]
    Unresolved,
    #[error("bpx error: {0}")]
    Bpx(bpx::shader::error::Error)
}

fn get_assembly_hash(file: &Path) -> Result<u64, Error> {
    let file = File::open(file).map_err(Error::Io)?;
    let pack = ShaderPack::open(BufReader::new(file)).map_err(Error::Bpx)?;
    Ok(pack.get_settings().assembly_hash)
}

fn link_single(path: &Path, new_assembly: u64) -> Result<(), Error> {
    let mut shader = ShaderPack::open(File::options().read(true).write(true).open(path).map_err(Error::Io)?).map_err(Error::Bpx)?;
    if shader.get_settings().assembly_hash != 0 {
        warn!("Shader pack {:?} is already linked, skipping...", path);
        return Ok(());
    }
    shader.set_assembly(new_assembly);
    let indices: Vec<usize> = shader.symbols().map_err(Error::Bpx)?.iter()
        .filter(|v| v.flags & FLAG_EXTERNAL != 0)
        .enumerate()
        .map(|(i, _)| i)
        .collect();
    let mut symbols = shader.symbols_mut().unwrap();
    for index in indices {
        symbols.get_mut(index).unwrap().flags |= FLAG_ASSEMBLY
    }
    shader.save().map_err(Error::Bpx)?;
    Ok(())
}

fn link(n_threads: usize, assembly: &Path, shaders: Vec<&Path>) -> Result<(), Error> {
    let new_assembly = get_assembly_hash(assembly)?;
    crossbeam::scope(|scope| {
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<(), Error>> = ThreadPool::new(n_threads);
        info!("Initialized thread pool with {} max thread(s)", n_threads);
        for shader in shaders {
            pool.send(&manager, move |_| link_single(shader, new_assembly));
            debug!("Dispatch shader pack {:?}", shader);
        }
        pool.reduce().map(|v| v.unwrap()).collect()
    }).unwrap()
}

pub fn run(config: Config) -> Result<(), Error> {
    info!("Loading and signing shader symbols...");
    let mut shader_tree = load_and_sign_symbols(config.n_threads, config.shaders.iter().map(|v| *v))
        .map_err(Error::Symbol)?;
    shader_tree.mass_set_internal();
    info!("Loading and signing assembly symbols...");
    let assembly_tree = load_and_sign_symbols(config.n_threads, [config.assembly].into_iter())
        .map_err(Error::Symbol)?;
    info!("Checking signatures against assembly...");
    check_signature_with_assembly(&mut shader_tree, &assembly_tree).map_err(crate::symbols::Error::Signing)
        .map_err(Error::Symbol)?;
    if !shader_tree.is_external() {
        error!("One or more external symbols were not found in shader assembly; linking cannot continue!");
        return Err(Error::Unresolved);
    }
    info!("Linking shaders...");
    link(config.n_threads, config.assembly, config.shaders)?;
    Ok(())
}

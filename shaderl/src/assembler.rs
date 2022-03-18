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

use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use bpx::core::builder::SectionHeaderBuilder;
use bpx::shader::ShaderPack;
use bpx::shader::symbol::{FLAG_EXTERNAL, FLAG_INTERNAL, FLAG_REGISTER};
use bpx::utils::hash;
use byteorder::{ByteOrder, LittleEndian};
use log::info;
use crate::symbols::{check_signature_with_assembly, load_and_sign_symbols};
use thiserror::Error;

pub struct Config<'a, I: Iterator<Item = &'a Path>> {
    pub n_threads: usize,
    pub debug: bool,
    pub output: Cow<'a, Path>,
    pub assembly: Option<&'a Path>,
    pub name: &'a str,
    pub shaders: I
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("symbol error: {0}")]
    Symbol(crate::symbols::Error),
    #[error("BPX shader error: {0}")]
    Shader(bpx::shader::error::Error),
    #[error("BPX core error: {0}")]
    Core(bpx::core::error::Error),
    #[error("BPX serialization error: {0}")]
    Serde(bpx::sd::serde::Error),
    #[error("section open error: {0}")]
    SectionOpen(bpx::core::error::OpenError)
}

fn get_assembly_hash(file: &Path) -> Result<u64, Error> {
    let file = File::open(file).map_err(Error::Io)?;
    let pack = ShaderPack::open(BufReader::new(file)).map_err(Error::Shader)?;
    Ok(pack.get_settings().assembly_hash)
}

pub fn run<'a>(config: Config<'a, impl Iterator<Item = &'a Path>>) -> Result<(), Error> {
    info!("Assembling '{}'...", config.name);
    let file = File::create(&config.output).map_err(Error::Io)?;
    info!("Loading and signing shader symbols...");
    let mut shader_tree = load_and_sign_symbols(config.n_threads, config.shaders)
        .map_err(Error::Symbol)?;
    shader_tree.mass_set_internal();
    info!("Loading and signing parent assembly symbols...");
    let assembly_tree = config.assembly.map(|v| load_and_sign_symbols(config.n_threads, [v].into_iter()))
        .transpose().map_err(Error::Symbol)?;
    if let Some(assembly) = &assembly_tree {
        info!("Checking signatures against parent assembly...");
        check_signature_with_assembly(&mut shader_tree, assembly).map_err(crate::symbols::Error::Signing)
            .map_err(Error::Symbol)?;
    }
    //Prepare the symbol tree to be written into an assembly.
    info!("Aligning symbol references...");
    shader_tree.align_references();
    info!("Writing symbols...");
    let mut pack = ShaderPack::create(BufWriter::new(file),
                                      bpx::shader::Builder::new()
                                          .ty(bpx::shader::Type::Assembly)
                                          .target(bpx::shader::Target::Any)
                                          .assembly(hash(config.name)));
    let mut symbols = pack.symbols_mut().unwrap();
    for sym in shader_tree.iter() {
        let mut builder = bpx::shader::symbol::Builder::new(sym.name());
        builder.ty(sym.info().ty);
        if sym.info().flags & FLAG_REGISTER != 0 {
            builder.register(sym.info().register);
        }
        if sym.info().flags & FLAG_INTERNAL != 0 {
            builder.internal();
        }
        if sym.info().flags & FLAG_EXTERNAL != 0 {
            builder.external();
        }
        if let Some(ptr) = sym.ext_data() {
            builder.extended_data(ptr.to_bpx(config.debug).map_err(Error::Serde)?);
        }
        symbols.create(builder).map_err(Error::Shader)?;
    }
    pack.save().map_err(Error::Shader)?;
    if let Some(assembly) = config.assembly {
        info!("Writing parent assembly hash...");
        let mut inner = pack.into_inner();
        { //Rust is garbage too stupid to see that inner is not used when save is called!
            let hash = get_assembly_hash(assembly)?;
            let handle = inner.sections_mut().create(SectionHeaderBuilder::new().ty(0xFD));
            let mut writer = inner.sections().open(handle).map_err(Error::SectionOpen)?;
            let mut buf = [0; 8];
            LittleEndian::write_u64(&mut buf, hash);
            writer.write_all(&buf).map_err(Error::Io)?;
        }
        inner.save().map_err(Error::Core)?;
    }
    info!("Generated assembly '{}' and saved to {:?}", config.name, config.output);
    Ok(())
}

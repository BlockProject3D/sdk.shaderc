// Copyright (c) 2021, BlockProject 3D
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

use std::io::Write;
use bpx::decoder::IoBackend;
use bpx::package::object::ObjectHeader;
use bpx::package::PackageDecoder;
use bpx::shader::Stage;
use bpx::table::{ItemTable, NameTable};
use sal::preprocessor::Handler;
use bpx::macros::impl_err_conversion;
use bpx::package::utils::unpack_memory;
use bpx::utils::OptionExtension;
use log::{debug, info, warn};
use crate::targets::basic::shaderlib::ShaderLib;

#[derive(Debug)]
pub enum Error
{
    Io(std::io::Error),
    UnknownStage(String),
    ShaderLib(crate::targets::basic::shaderlib::Error),
    NullInclude,
    IncludeNotFound(String)
}

impl_err_conversion!(
    Error {
        std::io::Error => Io,
        crate::targets::basic::shaderlib::Error => ShaderLib
    }
);

pub struct BasicPreprocessor<TBackend: IoBackend>
{
    sal_code: Vec<u8>,
    includes: Vec<Box<[u8]>>,
    src_code: Vec<String>,
    shader_libs: Vec<ShaderLib<TBackend>>,
    stage: Option<Stage>,
    line_is_directive: bool
}

impl<TBackend: IoBackend> BasicPreprocessor<TBackend>
{
    pub fn new(libs: Vec<PackageDecoder<TBackend>>) -> Self
    {
        Self {
            sal_code: Vec::new(),
            includes: Vec::new(),
            src_code: Vec::new(),
            shader_libs: libs.into_iter().map(|l| ShaderLib::new(l)).collect(),
            stage: None,
            line_is_directive: false
        }
    }
}

impl<TBackend: IoBackend> Handler for BasicPreprocessor<TBackend>
{
    type Error = Error;

    fn directive(&mut self, name: &str, value: Option<&str>) -> Result<(), Self::Error>
    {
        match name {
            "stage" => {
                let value = value.unwrap_or("");
                self.stage = Some(match value {
                    "vertex" =>  Stage::Vertex,
                    "hull" => Stage::Hull,
                    "domain" => Stage::Domain,
                    "geometry" => Stage::Geometry,
                    "pixel" => Stage::Pixel,
                    _ => return Err(Error::UnknownStage(value.into()))
                });
            },
            "include" => {
                let value = value.ok_or_else(|| Error::NullInclude)?;
                let mut flag = false;
                for v in &mut self.shader_libs {
                    if let Some(obj) = v.try_load(value)? {
                        self.includes.push(obj.into_boxed_slice());
                        flag = true;
                        debug!("Successfully resolved include {}", value);
                    }
                }
                if !flag {
                    return Err(Error::IncludeNotFound(value.into()));
                }
            },
            _ => return Ok(())
        };
        self.line_is_directive = true;
        Ok(())
    }

    fn sal_code(&mut self, content: &str) -> Result<(), Self::Error>
    {
        self.sal_code.write_all(content.as_bytes())?;
        Ok(())
    }

    fn code_line(&mut self, mut line: String) -> Result<(), Self::Error>
    {
        if self.line_is_directive {
            line.insert_str(0, "//");
        }
        self.src_code.push(line);
        Ok(())
    }
}

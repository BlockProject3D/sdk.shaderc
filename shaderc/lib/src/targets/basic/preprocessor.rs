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

use std::{io::Write, path::Path};
use std::fmt::{Display, Formatter};

use bpx::{macros::impl_err_conversion, shader::Stage};
use log::{debug, trace};
use bp3d_sal::preprocessor::Handler;

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

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::UnknownStage(s) => write!(f, "unknown shader stage '{}'", s),
            Error::ShaderLib(e) => write!(f, "error in shader lib: {}", e),
            Error::NullInclude => f.write_str("include does not have a value"),
            Error::IncludeNotFound(i) => write!(f, "include '{}' not found", i)
        }
    }
}

pub struct BasicPreprocessor<'a>
{
    pub sal_code: Vec<u8>,
    pub includes: Vec<(String, Box<[u8]>)>,
    pub src_code: Vec<String>,
    shader_libs: Vec<ShaderLib<'a>>,
    pub stage: Option<Stage>,
    line_is_directive: bool,
    using_sal: bool
}

impl<'a> BasicPreprocessor<'a>
{
    pub fn new(libs: &Vec<&'a Path>) -> Self
    {
        Self {
            sal_code: Vec::new(),
            includes: Vec::new(),
            src_code: Vec::new(),
            shader_libs: libs.into_iter().map(|l| ShaderLib::new(l)).collect(),
            stage: None,
            line_is_directive: false,
            using_sal: false
        }
    }
}

impl<'a> Handler for BasicPreprocessor<'a>
{
    type Error = Error;

    fn directive(&mut self, name: &str, value: Option<&str>) -> Result<(), Self::Error>
    {
        debug!("Found directive #{} {:?}", name, value);
        match name {
            "stage" => {
                let value = value.unwrap_or("");
                self.stage = Some(match value {
                    "vertex" => Stage::Vertex,
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
                        self.includes.push((value.into(), obj.into_boxed_slice()));
                        flag = true;
                        debug!("Successfully resolved include {}", value);
                    }
                }
                if !flag {
                    return Err(Error::IncludeNotFound(value.into()));
                }
            },
            "sal" => self.using_sal = !self.using_sal,
            _ => return Ok(())
        };
        self.line_is_directive = true;
        Ok(())
    }

    fn sal_code(&mut self, content: &str) -> Result<(), Self::Error>
    {
        trace!("SAL> {}", content);
        self.sal_code.write_all(content.as_bytes())?;
        self.sal_code.push(b'\n');
        Ok(())
    }

    fn code_line(&mut self, mut line: String) -> Result<(), Self::Error>
    {
        if self.line_is_directive || self.using_sal {
            line.insert_str(0, "//");
            self.line_is_directive = false;
        }
        if !self.using_sal {
            trace!("{}", line);
        }
        self.src_code.push(line);
        Ok(())
    }
}

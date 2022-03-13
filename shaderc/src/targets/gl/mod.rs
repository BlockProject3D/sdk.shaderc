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

mod core;
mod bindings;
mod bpx;
mod ext_data;

pub use self::core::EnvInfo;

use std::collections::BTreeMap;
use std::fs::File;
use ::bpx::shader::Stage;
use log::info;
use crate::options::{Args, Error};
use crate::targets::basic::{ShaderStage, Target};
use crate::targets::gl::bindings::{gl_relocate_bindings, gl_test_bindings};
use crate::targets::gl::bpx::BpxWriter;
use crate::targets::gl::core::ShaderBytes;

use self::core::Symbols;
use self::core::compile_stages;
use self::core::gl_link_shaders;

pub struct GlTarget
{
    env: EnvInfo,
    bpx_target: ::bpx::shader::Target
}

impl GlTarget {
    pub fn new(env: EnvInfo, bpx_target: ::bpx::shader::Target) -> GlTarget {
        GlTarget {
            env,
            bpx_target
        }
    }
}

impl Target for GlTarget {
    type CompileOutput = (Symbols, Vec<ShaderBytes>);

    fn relocate_bindings(&self, stages: &mut BTreeMap<Stage, ShaderStage>) -> Result<(), Error> {
        gl_relocate_bindings(stages);
        Ok(())
    }

    fn test_bindings(&self, stages: &BTreeMap<Stage, ShaderStage>) -> Result<(), Error> {
        gl_test_bindings(stages)
    }

    fn compile_link(&self, args: &Args, stages: BTreeMap<Stage, ShaderStage>) -> Result<Self::CompileOutput, Error> {
        rglslang::main(|| {
            info!("Compiling shaders...");
            let output = compile_stages(&self.env, &args, stages)?;
            info!("Linking shaders...");
            gl_link_shaders(&args, output)
        })
    }

    fn write_finish(&self, args: &Args, (symbols, shaders): Self::CompileOutput) -> Result<(), Error> {
        let mut bpx = BpxWriter::new(File::create(args.output)?, self.bpx_target, args.debug);
        bpx.write_symbols(symbols)?;
        bpx.write_shaders(shaders)?;
        bpx.save()?;
        Ok(())
    }
}

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

pub mod preprocessor;
pub mod shaderlib;
pub mod useresolver;
mod shader_to_sal;
mod sal_compiler;

use std::collections::BTreeMap;
use bpx::shader::Stage;
use log::info;
pub use shader_to_sal::*;
pub use sal_compiler::*;
use crate::options::{Args, Error};

pub trait Target
{
    type CompileOutput;

    fn pre_process(&self, args: &Args) -> Result<BTreeMap<Stage, ShaderStage>, Error> {
        info!("Running initial shader decomposition phase...");
        let shaders = decompose_pass(&args)?;
        info!("Merging shader stages");
        let stages = merge_stages(shaders)?;
        info!("Testing SAL symbols...");
        test_symbols(&stages)?;
        Ok(stages)
    }

    fn relocate_bindings(&self, stages: &mut BTreeMap<Stage, ShaderStage>) -> Result<(), Error>;

    fn test_bindings(&self, stages: &BTreeMap<Stage, ShaderStage>) -> Result<(), Error>;

    fn compile_link(&self, args: &Args, stages: BTreeMap<Stage, ShaderStage>) -> Result<Self::CompileOutput, Error>;

    fn write_finish(&self, args: &Args, out: Self::CompileOutput) -> Result<(), Error>;

    fn run(&self, args: &Args) -> Result<(), Error> {
        info!("Applying pre-processor...");
        let mut stages = self.pre_process(args)?;
        info!("Applying binding relocations...");
        self.relocate_bindings(&mut stages)?;
        info!("Testing binding relocations...");
        self.test_bindings(&stages)?;
        info!("Compiling and linking...");
        let out = self.compile_link(args, stages)?;
        info!("Writing {}...", args.output.display());
        self.write_finish(args, out)?;
        info!("Shader pack built: {}", args.output.display());
        Ok(())
    }
}

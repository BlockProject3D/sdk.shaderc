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

use log::info;
use crate::options::{Args, Error};
use crate::targets::basic::{decompose_pass, merge_stages, test_symbols};
use crate::targets::gl::{compile_stages, EnvInfo, gl_relocate_bindings, gl_test_bindings};

//TODO: At shader initialization, procedure for each binding:
// - glUseProgram(prog)
// - location = glGetUniformLocation(prog, binding_name)
// for constant buffers - glUniformBlockBinding(prog, location, binding)
// for objects - glUniform1i(location, binding)

pub fn build(args: Args) -> Result<(), Error>
{
    info!("Running initial shader decomposition phase...");
    let shaders = decompose_pass(&args)?;
    info!("Merging shader stages");
    let mut stages = merge_stages(shaders)?;
    info!("Testing SAL symbols...");
    test_symbols(&stages)?;
    info!("Applying binding relocations...");
    gl_relocate_bindings(&mut stages);
    info!("Testing binding relocations...");
    gl_test_bindings(&stages)?;
    rglslang::main(|| {
        let env = EnvInfo {
            gl_version_int: 400,
            gl_version_str: "4.0",
            explicit_bindings: false
        };
        info!("Compiling shaders...");
        compile_stages(&env, &args, stages).unwrap(); //We have a problem rust does not allow passing the error back to the build function
    });
    todo!()
}

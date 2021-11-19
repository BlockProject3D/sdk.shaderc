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

use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Deref;
use bp3d_threads::{ScopedThreadManager, ThreadPool};
use bpx::shader::Stage;
use log::{debug, info, warn};
use sal::ast::tree::{Attribute, BlendfuncStatement, PipelineStatement, Property, PropertyType, Statement, Struct};
use crate::options::{Args, Error};
use crate::targets::basic::{decompose_pass, decompose_statements, DecomposedShader, get_root_constants_layout, load_shader_to_sal, merge_stages, OrderedProp, relocate_bindings, ShaderStage, StmtDecomposition, test_bindings};
use crate::targets::sal_to_glsl::translate_sal_to_glsl;

fn compile_stages(args: &Args, stages: HashMap<Stage, ShaderStage>) -> Result<(), Error>
{
    let root_constants_layout = get_root_constants_layout(&stages)?;
    let root = crossbeam::scope(|scope| {
        let mut root = Vec::new();
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<(), Error>> = ThreadPool::new(args.n_threads);
        info!("Initialized thread pool with {} max thread(s)", args.n_threads);
        for (stage, shader) in &stages {
            pool.dispatch(&manager, |_| {
                debug!("Translating SAL AST for stage {:?} to GLSL for OpenGL 4.0...", *stage);
                let glsl = translate_sal_to_glsl(&root_constants_layout, &shader.statements)?;
                info!("Translated GLSL: \n{}", glsl);
                Ok(())
            });
            debug!("Dispatch stage {:?}", stage);
        }
        pool.join().unwrap();
        while let Some(res) = pool.poll() {
            root.push(res);
        }
        root
    }).unwrap();
    for v in root {
        v?;
    }
    Ok(())
}

pub fn build(args: Args) -> Result<(), Error>
{
    info!("Running initial shader decomposition phase...");
    let shaders = decompose_pass(&args)?;
    info!("Merging shader stages");
    let mut stages = merge_stages(shaders)?;
    info!("Applying binding relocations...");
    relocate_bindings(&mut stages);
    info!("Testing binding relocations...");
    test_bindings(&stages)?;
    //relocate_bindings(&mut shaders);
    info!("Compiling shaders...");
    compile_stages(&args, stages)?;
    todo!()
}

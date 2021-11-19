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
use std::collections::{BTreeSet, HashSet};
use std::ops::Deref;
use bp3d_threads::{ScopedThreadManager, ThreadPool};
use log::{debug, info};
use sal::ast::tree::{BlendfuncStatement, PipelineStatement, Property, PropertyType, Statement, Struct};
use crate::options::{Args, Error};
use crate::targets::basic::{decompose_statements, load_shader_to_sal, OrderedProp, StmtDecomposition};
use crate::targets::sal_to_glsl::translate_sal_to_glsl;

struct DecomposedShader
{
    name: String,
    statements: StmtDecomposition,
    strings: Vec<rglslang::shader::Part>
}

fn decompose_pass(args: &Args) -> Result<Vec<DecomposedShader>, Error>
{
    let root = crossbeam::scope(|scope| {
        let mut root = Vec::new();
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<DecomposedShader, Error>> = ThreadPool::new(args.n_threads);
        info!("Initialized thread pool with {} max thread(s)", args.n_threads);
        for unit in &args.units {
            pool.dispatch(&manager, |_| {
                debug!("Loading SAL for shader unit {:?}...", *unit);
                let res = load_shader_to_sal(unit, &args)?;
                debug!("Decomposing SAL AST for shader unit {:?}...", *unit);
                let sal = decompose_statements(res.statements)?;
                let decomposed = DecomposedShader {
                    name: res.name,
                    statements: sal,
                    strings: res.strings
                };
                /*debug!("Translating SAL AST for shader unit {:?} to GLSL for OpenGL 4.0...", *unit);
                let glsl = translate_sal_to_glsl(&sal)?;
                info!("Translated GLSL: \n{}", glsl);*/
                Ok(decomposed)
            });
            debug!("Dispatch shader unit {:?}", unit);
        }
        pool.join().unwrap();
        while let Some(res) = pool.poll() {
            root.push(res);
        }
        root
    }).unwrap();
    let mut vec = Vec::new();
    for v in root {
        vec.push(v?);
    }
    Ok(vec)
}

pub fn build(args: Args) -> Result<(), Error>
{
    info!("Running initial shader decomposition phase...");
    let shaders = decompose_pass(&args)?;
    info!("Applying relocations...");
    todo!()
}

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

fn translate_property(p: &Property) -> String
{
    let ptype: Cow<str> = match p.ptype {
        PropertyType::Scalar(s) => s.get_name().into(),
        PropertyType::Vector(v) => format!("vec{}{}", v.size, v.item.get_char()).into(),
        PropertyType::Matrix(m) => format!("mat{}{}", m.size, m.item.get_char()).into(),
        PropertyType::Sampler => "".into(),
        PropertyType::Texture2D(_) => "sampler2D".into(),
        PropertyType::Texture3D(_) => "sampler3D".into(),
        PropertyType::Texture2DArray(_) => "sampler2DArray".into(),
        PropertyType::TextureCube(_) => "samplerCube".into()
    };
    if &ptype == "" {
        return String::default()
    }
    format!("{} {};", ptype, p.pname)
}

fn translate_cbuffer(binding: usize, s: &Struct) -> String
{
    let mut str = format!("layout (binding = {}, std140) uniform {} {{", binding, s.name);
    for v in &s.props {
        let prop = Property {
            pattr: None,
            pname: [&*s.name, &*v.pname].join("_"),
            ptype: v.ptype
        };
        str.push_str(&translate_property(&prop));
    }
    str.push_str("};");
    str
}

fn translate_vformat(s: &Struct) -> String
{
    let mut str= String::new();
    for (loc, v) in s.props.iter().enumerate() {
        let prop = Property {
            pattr: None,
            pname: [&*s.name, &*v.pname].join("_"),
            ptype: v.ptype
        };
        str.push_str(&format!("layout (location = {}) in {}", loc, translate_property(&prop)));
    }
    str
}

fn translate_outputs(outputs: &BTreeSet<OrderedProp>) -> Result<String, Error>
{
    let mut str= String::new();
    let mut set = HashSet::new();
    for v in outputs.iter() {
        if !set.insert(v.get_native_order()) {
            return Err(Error::from(format!("multiple definition of output slot {}", v.get_native_order())))
        }
        str.push_str(&format!("layout (location = {}) out {}", v.get_native_order(), translate_property(v.inner)));
    }
    Ok(str)
}

fn translate_root_consts(consts: &BTreeSet<OrderedProp>) -> String
{
    let mut str = String::from("layout (binding = 0, std140) uniform __Root {");
    for v in consts {
        str.push_str(&translate_property(v.inner));
    }
    str.push_str("};");
    str
}

fn translate_sal_to_glsl(sal: &StmtDecomposition) -> Result<String, Error>
{
    let vformat = sal.vformat.map(|s| translate_vformat(s)).unwrap_or_default();
    let constants = translate_root_consts(&sal.root_constants);
    let outputs = translate_outputs(&sal.outputs)?;
    let cbuffers: Vec<String> = sal.cbuffers.iter().enumerate().map(|(i, s)| translate_cbuffer(i + 1, s)).collect();
    let cbuffers = cbuffers.join("\n");
    let objects: Vec<String> = sal.objects.iter().enumerate().map(|(i, p)| format!("layout (binding = {}) uniform {}", i, translate_property(p))).collect();
    let objects = objects.join("\n");
    debug!("translated vertex format: {}", vformat);
    debug!("translated root constants: {}", constants);
    debug!("translated outputs: {}", outputs);
    debug!("translated constant buffers: {}", cbuffers);
    debug!("translated objects: {}", objects);
    let output = [vformat, constants, outputs, cbuffers, objects].join("\n");
    Ok(output)
}

pub fn build(args: Args) -> Result<(), Error>
{
    let mut root = Vec::new();
    crossbeam::scope(|scope| {
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<(), Error>> = ThreadPool::new(args.n_threads);
        info!("Initialized thread pool with {} max thread(s)", args.n_threads);
        for unit in &args.units {
            pool.dispatch(&manager, |_| {
                debug!("Loading SAL for shader unit {:?}...", *unit);
                let res = load_shader_to_sal(unit, &args)?;
                debug!("Decomposing SAL AST for shader unit {:?}...", *unit);
                let sal = decompose_statements(&res.statements)?;
                debug!("Translating SAL AST for shader unit {:?} to GLSL for OpenGL 4.0...", *unit);
                let glsl = translate_sal_to_glsl(&sal)?;
                Ok(())
            });
            debug!("Dispatch shader unit {:?}", unit);
        }
        pool.join().unwrap();
        while let Some(res) = pool.poll() {
            root.push(res);
        }
    }).unwrap();
    for v in root {
        v?;
    }
    todo!()
}

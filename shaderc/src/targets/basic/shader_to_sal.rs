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

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;
use bpx::shader::Stage;
use log::warn;
use sal::ast::tree::{BlendfuncStatement, PipelineStatement, Property, PropertyType, Statement, Struct};
use sal::utils::auto_lexer_parser;
use crate::options::{Args, Error, ShaderUnit};
use crate::targets::basic::preprocessor::BasicPreprocessor;
use crate::targets::basic::shaderlib::ShaderLib;
use crate::targets::basic::useresolver::BasicUseResolver;
use sal::preprocessor;

pub struct ShaderToSal
{
    pub strings: Vec<rglslang::shader::Part>,
    pub statements: Vec<Statement>
}

fn shader_sal_stage<T: BufRead>(name: String, content: T, args: &Args) -> Result<ShaderToSal, Error>
{
    let mut result = ShaderToSal {
        strings: Vec::new(),
        statements: Vec::new()
    };
    let mut preprocessor = BasicPreprocessor::new(&args.libs);
    preprocessor::run(content, &mut preprocessor)?;
    let stage = preprocessor.stage.unwrap_or_else(|| {
        warn!("No shader stage specified in shader file, assuming this is a vertex shader by default");
        Stage::Vertex
    });
    for (name, header) in preprocessor.includes {
        let data = shader_sal_stage(name,header.deref(), args)?;
        result.strings.extend(data.strings);
        result.statements.extend(data.statements);
    }
    result.statements.extend(auto_lexer_parser(&preprocessor.sal_code, BasicUseResolver::new(&args.libs))?);
    result.strings.push(rglslang::shader::Part::new_with_name(preprocessor.src_code.join("\n"), name));
    Ok(result)
}

pub fn load_shader_to_sal(unit: &ShaderUnit, args: &Args) -> Result<ShaderToSal, Error>
{
    let mut libs: Vec<ShaderLib> = args.libs.iter().map(|v| ShaderLib::new(*v)).collect();
    match unit {
        ShaderUnit::Path(path) => {
            let reader = BufReader::new(File::open(path)?);
            shader_sal_stage(path.to_string_lossy().into_owned(),reader, args)
        },
        ShaderUnit::Injected(vname) => {
            for v in &mut libs {
                if let Some(data) = v.try_load(vname)? {
                    return shader_sal_stage(String::from(*vname), data.as_slice(), args);
                }
            }
            Err(Error::from(format!("unable to locate injected shader '{}'", vname)))
        }
    }
}

pub struct OrderedProp<'a>
{
    pub inner: &'a Property,
    index: u64
}

impl<'a> OrderedProp<'a>
{
    pub fn get_native_order(&self) -> u32
    {
        let order = self.inner.pattr.as_ref().map(|s| s.get_order()).unwrap_or_default().unwrap_or(0);
        order
    }
}

impl<'a> PartialEq<Self> for OrderedProp<'a>
{
    fn eq(&self, other: &Self) -> bool
    {
        let order = self.inner.pattr.as_ref().map(|s| s.get_order()).unwrap_or_default().unwrap_or(0);
        let order1 = other.inner.pattr.as_ref().map(|s| s.get_order()).unwrap_or_default().unwrap_or(0);
        order == order1 && self.index == other.index
    }
}

impl<'a> PartialOrd for OrderedProp<'a>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}

impl<'a> Eq for OrderedProp<'a> {}

impl<'a> Ord for OrderedProp<'a>
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        let order = self.inner.pattr.as_ref().map(|s| s.get_order()).unwrap_or_default().unwrap_or(0);
        let order1 = other.inner.pattr.as_ref().map(|s| s.get_order()).unwrap_or_default().unwrap_or(0);
        if order < order1 && self.index < other.index {
            Ordering::Less
        } else if order > order1 && self.index > other.index {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

pub struct StmtDecomposition<'a>
{
    pub root_constants: BTreeSet<OrderedProp<'a>>, //Root constants/push constants, emulated by global uniform buffer in GL targets
    pub outputs: BTreeSet<OrderedProp<'a>>, //Fragment shader outputs/render target outputs
    pub objects: Vec<&'a Property>, //Samplers and textures
    pub cbuffers: Vec<&'a Struct>,
    pub vformat: Option<&'a Struct>,
    pub pipeline: Option<&'a PipelineStatement>,
    pub blendfuncs: Vec<&'a BlendfuncStatement>
}

pub fn decompose_statements<'a>(stmts: &'a Vec<Statement>) -> Result<StmtDecomposition<'a>, Error>
{
    let mut rcounter: u64 = 0;
    let mut ocounter: u64 = 0;
    let mut root_constants = BTreeSet::new();
    let mut outputs= BTreeSet::new();
    let mut objects = Vec::new();
    let mut cbuffers= Vec::new();
    let mut vformat = None;
    let mut pipeline = None;
    let mut blendfuncs= Vec::new();
    let mut insert_root_const = |p: &'a Property| {
        assert!(root_constants.insert(OrderedProp {
            inner: p,
            index: rcounter
        }));
        rcounter += 1;
    };
    let mut insert_output = |p: &'a Property| {
        assert!(outputs.insert(OrderedProp {
            inner: p,
            index: ocounter
        }));
        ocounter += 1;
    };
    for v in stmts {
        match v {
            Statement::Constant(p) => {
                match p.ptype {
                    PropertyType::Scalar(_) => insert_root_const(p),
                    PropertyType::Vector(_) => insert_root_const(p),
                    PropertyType::Matrix(_) => insert_root_const(p),
                    _ => objects.push(p)
                };
            }
            Statement::ConstantBuffer(s) => cbuffers.push(s),
            Statement::Output(o) => insert_output(o),
            Statement::VertexFormat(s) => {
                if vformat.is_some() {
                    return Err(Error::new("only 1 vertex format is allowed per shader"));
                }
                vformat = Some(s);
            },
            Statement::Pipeline(p) => {
                if pipeline.is_some() {
                    return Err(Error::new("only 1 pipeline is allowed per shader"));
                }
                pipeline = Some(p);
            },
            Statement::Blendfunc(b) => blendfuncs.push(b),
            Statement::Noop => (),
        }
    }
    Ok(StmtDecomposition {
        root_constants,
        objects,
        outputs,
        cbuffers,
        vformat,
        pipeline,
        blendfuncs
    })
}

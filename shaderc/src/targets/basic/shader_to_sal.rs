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

use std::cell::Cell;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;
use bp3d_threads::{ScopedThreadManager, ThreadPool};
use bpx::shader::Stage;
use log::{debug, info, trace, warn};
use bp3d_sal::ast::tree::{ArrayItemType, Attribute, BlendfuncStatement, PipelineStatement, Property, PropertyType, Statement, Struct};
use bp3d_sal::ast::Visitor;
use bp3d_sal::utils::auto_lexer_parser;
use crate::targets::basic::preprocessor::BasicPreprocessor;
use crate::targets::basic::shaderlib::ShaderLib;
use crate::targets::basic::useresolver::BasicUseResolver;
use bp3d_sal::preprocessor;
use crate::config::{Config, Unit};
use crate::targets::basic::ast::Ast;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VisitorError
{
    #[error("only 1 vertex format is allowed per shader")]
    DuplicateVertexFormat,
    #[error("only 1 pipeline definition is allowed per shader")]
    DuplicatePipeline,
    #[error("error while resolving use statement: {0}")]
    Use(crate::targets::basic::useresolver::Error)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("sal error: {0}")]
    Sal(bp3d_sal::utils::AutoError<usize, VisitorError>),
    #[error("shader lib error: {0}")]
    ShaderLib(crate::targets::basic::shaderlib::Error),
    #[error("unable to locate injected shader")]
    InjectionNotFound,
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("preprocessor error: {0}")]
    Preprocessor(crate::targets::basic::preprocessor::Error)
}

pub type BasicAst = Ast<
    Slot<Property<usize>>, Slot<Property<usize>>, Slot<Property<usize>>,
    Struct<usize>, Struct<usize>, Slot<Struct<usize>>, Struct<usize>
>;

impl BasicAst {
    fn insert_struct(&mut self, mut val: Struct<usize>, src: &mut BasicAst) -> Struct<usize> {
        for p in &mut val.props {
            match p.ptype {
                PropertyType::StructRef(v) => {
                    let st = src.remove_packed_struct(v);
                    let obj = self.insert_struct(st, src);
                    let newid = self.push_packed_struct(obj.name.clone(), obj);
                    p.ptype = PropertyType::StructRef(newid);
                },
                PropertyType::Array(v) => {
                    match v.item {
                        ArrayItemType::StructRef(v) => {
                            let st = src.remove_packed_struct(v);
                            let obj = self.insert_struct(st, src);
                            let newid = self.push_packed_struct(obj.name.clone(), obj);
                            p.ptype = PropertyType::StructRef(newid);
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
        }
        val
    }

    pub fn extend(&mut self, mut other: BasicAst) {
        if other.root_constants_layout.is_some() && self.root_constants_layout.is_some() {
            unsafe { //Rust has just lost the concept of expressions...
                warn!("Overwriting root constants layout with '{}'", other.root_constants_layout.as_ref().unwrap_unchecked().name);
            }
        }
        if let Some(v) = other.root_constants_layout.take() {
            let v = self.insert_struct(v, &mut other);
            self.root_constants_layout = Some(v);
        }
        let cbuffers = std::mem::replace(&mut other.cbuffers, Vec::new());
        for mut v in cbuffers {
            v.inner = self.insert_struct(v.inner, &mut other);
            self.cbuffers.push(v);
        }
        if other.vformat.is_some() && self.vformat.is_some() {
            unsafe { //Rust has just lost the concept of expressions...
                warn!("Overwriting vertex format with '{}'", other.vformat.as_ref().unwrap_unchecked().name);
            }
        }
        if other.vformat.is_some() {
            self.vformat = other.vformat;
        }
        if other.pipeline.is_some() && self.pipeline.is_some() {
            unsafe { //Rust has just lost the concept of expressions...
                warn!("Overwriting pipeline description with '{}'", other.pipeline.as_ref().unwrap_unchecked().name);
            }
        }
        if other.pipeline.is_some() {
            self.pipeline = other.pipeline;
        }
        self.blendfuncs.extend(other.blendfuncs);
        self.objects.extend(other.objects);
        self.root_constants.extend(other.root_constants);
        self.outputs.extend(other.outputs);
    }
}

pub struct AstVisitor<'a> {
    resolver: BasicUseResolver<'a>
}

impl<'a> Visitor<BasicAst> for AstVisitor<'a> {
    type Error = VisitorError;

    fn visit_constant(&mut self, ast: &mut BasicAst, val: Property<usize>) -> Result<(), Self::Error> {
        trace!("Visit constant: {}", val.pname);
        match val.ptype {
            PropertyType::Scalar(_) => ast.root_constants.push(Slot::new(val)),
            PropertyType::Vector(_) => ast.root_constants.push(Slot::new(val)),
            PropertyType::Matrix(_) => ast.root_constants.push(Slot::new(val)),
            _ => ast.objects.push(Slot::new(val))
        };
        Ok(())
    }

    fn visit_output(&mut self, ast: &mut BasicAst, val: Property<usize>) -> Result<(), Self::Error> {
        trace!("Visit output: {}", val.pname);
        let slot = Slot::new(val);
        if let Some(attr) = &slot.inner.pattr {
            if let Attribute::Order(id) = attr {
                slot.slot.set(*id);
                slot.external.set(true);
            }
        }
        ast.outputs.push(slot);
        Ok(())
    }

    fn visit_constant_buffer(&mut self, ast: &mut BasicAst, val: Struct<usize>) -> Result<(), Self::Error> {
        trace!("Visit constant buffer: {}", val.name);
        if let Some(attr) = &val.attr {
            match attr {
                Attribute::Order(o) => {
                    if *o == 0 {
                        trace!("Constant buffer '{}' is root", val.name);
                        ast.root_constants_layout = Some(val);
                    } else {
                        trace!("Constant buffer '{}' is at slot #{}", val.name, o);
                        ast.cbuffers.push(Slot::new(val))
                    }
                }
                Attribute::Pack => {
                    trace!("Constant buffer '{}' is a packed struct", val.name);
                    ast.push_packed_struct(val.name.clone(), val);
                }
                _ => ()
            }
        } else {
            trace!("Constant buffer '{}' is unbounded", val.name);
            ast.cbuffers.push(Slot::new(val))
        }
        Ok(())
    }

    fn visit_vertex_format(&mut self, ast: &mut BasicAst, val: Struct<usize>) -> Result<(), Self::Error> {
        trace!("Visit vertex format: {}", val.name);
        if ast.vformat.is_some() {
            return Err(VisitorError::DuplicateVertexFormat);
        }
        ast.vformat = Some(val);
        Ok(())
    }

    fn visit_pipeline(&mut self, ast: &mut BasicAst, val: PipelineStatement) -> Result<(), Self::Error> {
        trace!("Visit pipeline description: {}", val.name);
        if ast.pipeline.is_some() {
            return Err(VisitorError::DuplicatePipeline);
        }
        ast.pipeline = Some(val);
        Ok(())
    }

    fn visit_blendfunc(&mut self, ast: &mut BasicAst, val: BlendfuncStatement) -> Result<(), Self::Error> {
        trace!("Visit blend function description: {}", val.name);
        ast.blendfuncs.push(val);
        Ok(())
    }

    fn visit_noop(&mut self, _: &mut BasicAst) -> Result<(), Self::Error> {
        trace!("Visit noop");
        //Do nothing.
        Ok(())
    }

    fn visit_use(&mut self, ast: &mut BasicAst, module: String, member: String) -> Result<(), Self::Error> {
        trace!("Visit use: {}::{}", module, member);
        let (stmt, mut ast1) = self.resolver.resolve(module, member)
            .map_err(VisitorError::Use)?;
        match stmt {
            Statement::Constant(v) => self.visit_constant(ast, v),
            Statement::ConstantBuffer(v) => {
                let v = ast.insert_struct(v, &mut ast1);
                self.visit_constant_buffer(ast, v)
            },
            Statement::Output(v) => self.visit_output(ast, v),
            Statement::VertexFormat(v) => self.visit_vertex_format(ast, v),
            Statement::Pipeline(v) => self.visit_pipeline(ast, v),
            Statement::Blendfunc(v) => self.visit_blendfunc(ast, v),
            Statement::Noop => self.visit_noop(ast)
        }
    }
}

pub struct ShaderToSal
{
    pub name: String,
    pub strings: Vec<rglslang::shader::Part>,
    pub statements: BasicAst,
    pub stage: Stage
}

fn shader_sal_stage<T: BufRead>(name: String, content: T, config: &Config) -> Result<ShaderToSal, Error>
{
    let mut result = ShaderToSal {
        strings: Vec::new(),
        statements: BasicAst::new(),
        name: name.clone(),
        stage: Stage::Vertex
    };
    let mut preprocessor = BasicPreprocessor::new(&config.libs);
    preprocessor::run(content, &mut preprocessor).map_err(Error::Preprocessor)?;
    result.stage = preprocessor.stage.unwrap_or_else(|| {
        warn!("No shader stage specified in shader file, assuming this is a vertex shader by default");
        Stage::Vertex
    });
    for (name, header) in preprocessor.includes {
        let data = shader_sal_stage(name,header.deref(), config)?;
        result.strings.extend(data.strings);
        result.statements.extend(data.statements);
    }
    let ast = auto_lexer_parser(&preprocessor.sal_code, BasicAst::new(), AstVisitor { resolver: BasicUseResolver::new(&config.libs) })
        .map_err(Error::Sal)?;
    result.statements.extend(ast);
    result.strings.push(rglslang::shader::Part::new_with_name(preprocessor.src_code.join("\n"), name));
    Ok(result)
}

pub fn load_shader_to_sal(unit: &Unit, config: &Config) -> Result<ShaderToSal, Error>
{
    let mut libs: Vec<ShaderLib> = config.libs.iter().map(|v| ShaderLib::new(*v)).collect();
    match unit {
        Unit::Path(path) => {
            info!("Loading shader {:?}...", path);
            let reader = BufReader::new(File::open(path).map_err(Error::Io)?);
            shader_sal_stage(path.to_string_lossy().into_owned(),reader, config)
        },
        Unit::Injected(vname) => {
            info!("Loading injected shader {}...", vname);
            for v in &mut libs {
                if let Some(data) = v.try_load(vname).map_err(Error::ShaderLib)? {
                    return shader_sal_stage(String::from(*vname), data.as_slice(), config);
                }
            }
            Err(Error::InjectionNotFound)
        }
    }
}

pub fn load_pass(config: &Config) -> Result<Vec<ShaderToSal>, Error>
{
    crossbeam::scope(|scope| {
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<ShaderToSal, Error>> = ThreadPool::new(config.n_threads);
        info!("Initialized thread pool with {} max thread(s)", config.n_threads);
        for unit in &config.units {
            pool.send(&manager, |_| {
                debug!("Loading SAL AST for shader unit {:?}...", *unit);
                load_shader_to_sal(unit, &config)
            });
            debug!("Dispatch shader unit {:?}", unit);
        }
        pool.reduce().map(|v| v.unwrap()).collect()
    }).unwrap()
}

pub struct Slot<T>
{
    pub inner: T,
    pub slot: Cell<u32>,
    pub external: Cell<bool>
}

impl<T> Slot<T>
{
    pub fn new(t: T) -> Self
    {
        Self {
            inner: t,
            slot: Cell::new(0),
            external: Cell::new(false)
        }
    }
}

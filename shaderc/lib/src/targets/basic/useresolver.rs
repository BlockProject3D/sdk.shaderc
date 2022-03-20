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

use std::path::Path;

use log::debug;
use thiserror::Error;
use bp3d_sal::ast::tree::Statement;
use bp3d_sal::ast::tree::{Attribute, BlendfuncStatement, PipelineStatement, Property, Struct};
use bp3d_sal::ast::{AstBuilder, Visitor};
use bp3d_sal::lexer::Lexer;
use bp3d_sal::parser::error::ParserOrVisitor;
use bp3d_sal::parser::Parser;
use crate::targets::basic::BasicAst;

use crate::targets::basic::shaderlib::ShaderLib;

#[derive(Debug, Error)]
pub enum Error
{
    #[error("shader lib error: {0}")]
    ShaderLib(crate::targets::basic::shaderlib::Error),
    #[error("SAL lexer error: {0}")]
    Lexer(bp3d_sal::lexer::error::Error),
    #[error("SAL parser error: {0}")]
    Parser(bp3d_sal::parser::error::Error),
    #[error("SAL AST error: {0}")]
    Ast(bp3d_sal::ast::error::Error<usize, ()>),
    #[error("module not found '{0}'")]
    ModuleNotFound(String),
    #[error("member not found '{0}'")]
    MemberNotFound(String)
}

pub struct EarlyStopVisitor<'a> {
    member: &'a str
}

impl<'a> Visitor<BasicAst> for EarlyStopVisitor<'a> {
    type Error = Statement<usize>;

    fn visit_constant(&mut self, _: &mut BasicAst, val: Property<usize>) -> Result<(), Self::Error> {
        if val.pname == self.member {
            Err(Statement::Constant(val))
        } else {
            Ok(())
        }
    }

    fn visit_output(&mut self, _: &mut BasicAst, val: Property<usize>) -> Result<(), Self::Error> {
        if val.pname == self.member {
            Err(Statement::Output(val))
        } else {
            Ok(())
        }
    }

    fn visit_constant_buffer(&mut self, ast: &mut BasicAst, val: Struct<usize>) -> Result<(), Self::Error> {
        if val.name == self.member {
            Err(Statement::ConstantBuffer(val))
        } else {
            let is_packed = val.attr.as_ref().map(|v| v == &Attribute::Pack).unwrap_or_default();
            if is_packed {
                ast.push_packed_struct(val.name.clone(), val);
            }
            Ok(())
        }
    }

    fn visit_vertex_format(&mut self, _: &mut BasicAst, val: Struct<usize>) -> Result<(), Self::Error> {
        if val.name == self.member {
            Err(Statement::VertexFormat(val))
        } else {
            Ok(())
        }
    }

    fn visit_pipeline(&mut self, _: &mut BasicAst, val: PipelineStatement) -> Result<(), Self::Error> {
        if val.name == self.member {
            Err(Statement::Pipeline(val))
        } else {
            Ok(())
        }
    }

    fn visit_blendfunc(&mut self, _: &mut BasicAst, val: BlendfuncStatement) -> Result<(), Self::Error> {
        if val.name == self.member {
            Err(Statement::Blendfunc(val))
        } else {
            Ok(())
        }
    }

    fn visit_noop(&mut self, _: &mut BasicAst) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_use(&mut self, ast: &mut BasicAst, _: String, _: String) -> Result<(), Self::Error> {
        self.visit_noop(ast) //We don't support use statements in use contexts.
    }
}

pub struct BasicUseResolver<'a>
{
    shader_libs: Vec<ShaderLib<'a>>
}

impl<'a> BasicUseResolver<'a>
{
    pub fn new(libs: &Vec<&'a Path>) -> Self
    {
        Self {
            shader_libs: libs.into_iter().map(|l| ShaderLib::new(l)).collect()
        }
    }

    pub fn resolve(&mut self, module1: String, member: String) -> Result<(Statement<usize>, BasicAst), Error>
    {
        for v in &mut self.shader_libs {
            if let Some(module) = v.try_load(&module1).map_err(Error::ShaderLib)? {
                let mut lexer = Lexer::new();
                lexer.process(module.as_ref()).map_err(Error::Lexer)?;
                let mut parser = Parser::new(lexer);
                let mut builder = AstBuilder::new(BasicAst::new(), EarlyStopVisitor { member: &member });
                let ast = parser.parse(&mut builder);
                return match ast {
                    Ok(_) => Err(Error::MemberNotFound(member.clone())),
                    Err(err) => {
                        match err {
                            ParserOrVisitor::Parser(e) => Err(Error::Parser(e)),
                            ParserOrVisitor::Visitor(e) => {
                                match e {
                                    bp3d_sal::ast::error::Error::Type(e) => Err(Error::Ast(bp3d_sal::ast::error::Error::Type(e))),
                                    bp3d_sal::ast::error::Error::Value(e) => Err(Error::Ast(bp3d_sal::ast::error::Error::Value(e))),
                                    bp3d_sal::ast::error::Error::Visitor(stmt) => {
                                        debug!("Successfully resolved module {} with member {}", module1, member);
                                        Ok((stmt, builder.into_inner()))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return Err(Error::ModuleNotFound(module1));
    }
}

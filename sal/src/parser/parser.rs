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

use std::collections::VecDeque;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::parser::tree;

pub struct Parser
{
    tokens: VecDeque<(Token, usize, usize)>
}

impl Parser
{
    pub fn new(lexer: Lexer) -> Parser
    {
        return Parser {
            tokens: lexer.into_tokens()
        };
    }

    fn pop(&mut self, line: usize, col: usize) -> Result<(Token, usize, usize), String>
    {
        if let Some((tok, line, col)) = self.tokens.pop_front() {
            return Ok((tok, line, col));
        }
        return Err(format!(
            "[Shader Annotation Language] Unexpected EOF at line {}, column {}",
            line, col
        ));
    }

    fn try_parse_use(&mut self, (token, line, col): &(Token, usize, usize)) -> Result<Option<tree::Root>, String>
    {
        if token == &Token::Use {
            let (tok, line, col) = self.pop(*line, *col)?;
            match tok {
                Token::Identifier(module) => {
                    //pop_expect Colon
                    //pop_expect Colon
                    let (tok, line, col) = self.pop(line, col)?;
                    match tok {
                        Token::Identifier(member) => {
                            return Ok(Some(tree::Root::Use(tree::Use {
                                module,
                                member
                            })));
                        },
                        _ => return Err(format!(
                            "[Shader Annotation Language] Unexpected token, expected identifier but got {} at line {}, column {}",
                            &tok, line, col
                        ))
                    };
                },
                _ => return Err(format!(
                    "[Shader Annotation Language] Unexpected token, expected identifier but got {} at line {}, column {}",
                    &tok, line, col
                ))
            };
            /*if let Token::Namespace(module, item) = tok {
                /*let v: Vec<&str> = n.split("::").collect();
                if v.len() != 2 {
                    return Err(format!(
                        "[Shader Annotation Language] Bad namespace path format ('{}') at line {}, column {}",
                        &n, line, col
                    ));
                }*/
                let module = String::from(v[0]);
                let member = String::from(v[0]);
                return Ok(Some(tree::Root::Use(tree::Use {
                    module,
                    member
                })));
            }*/
            /*return Err(format!(
                "[Shader Annotation Language] Unexpected token, expected identifier but got {} at line {}, column {}",
                &tok, line, col
            ));*/
        }
        return Ok(None);
    }

    fn parse_property(&mut self, line: usize, col: usize) -> Result<tree::Property, String>
    {
        let (tok, line, col) = self.pop(line, col)?;
        if let Token::Identifier(t) = tok {
            let (tok, line, col) = self.pop(line, col)?;
            if let Token::Identifier(n) = tok {
                return Ok(tree::Property { pname: n, ptype: t });
            }
            return Err(format!(
                "[Shader Annotation Language] Unexpected token, expected identifier but got {} at line {}, column {}",
                &tok, line, col
            ));
        }
        /*if let Token::Namespace(t) = tok {
            let (tok, line, col) = self.pop(line, col)?;
            if let Token::Identifier(n) = tok {
                return Ok(tree::Property { pname: n, ptype: t });
            }
            return Err(format!(
                "[Shader Annotation Language] Unexpected token, expected identifier but got {} at line {}, column {}",
                &tok, line, col
            ));
        }*/
        return Err(format!(
            "[Shader Annotation Language] Unexpected token, expected identifier but got {} at line {}, column {}",
            &tok, line, col
        ));
    }

    fn try_parse_output(&mut self, (token, line, col): &(Token, usize, usize)) -> Result<Option<tree::Root>, String>
    {
        if token == &Token::Output {
            let prop = self.parse_property(*line, *col)?;
            return Ok(Some(tree::Root::Output(prop)));
        }
        return Ok(None);
    }

    fn check_block_end(&mut self, line: usize, col: usize) -> Result<bool, String>
    {
        if let Some((tok, _, _)) = self.tokens.front() {
            if tok == &Token::BlockEnd {
                self.pop(line, col)?;
                return Ok(true);
            }
        }
        return Ok(false);
    }

    fn parse_struct(&mut self, line: usize, col: usize) -> Result<tree::Struct, String>
    {
        let (tok, line, col) = self.pop(line, col)?;
        if tok == Token::Struct {
            let (tok, line, col) = self.pop(line, col)?;
            if let Token::Identifier(sname) = tok {
                let (tok, line, col) = self.pop(line, col)?;
                if tok == Token::BlockStart {
                    let mut v = Vec::new();
                    loop {
                        let prop = self.parse_property(line, col)?;
                        v.push(prop);
                        if self.check_block_end(line, col)? {
                            break;
                        }
                    }
                    return Ok(tree::Struct {
                        name: sname,
                        properties: v
                    });
                }
                return Err(format!(
                    "[Shader Annotation Language] Unexpected token, expected '{{' but got {} at line {}, column {}",
                    &tok, line, col
                ));
            }
            return Err(format!(
                "[Shader Annotation Language] Unexpected token, expected identifier but got {} at line {}, column {}",
                &tok, line, col
            ));
        }
        return Err(format!(
            "[Shader Annotation Language] Unexpected token, expected struct but got {} at line {}, column {}",
            &tok, line, col
        ));
    }

    fn try_parse_const(&mut self, (token, line, col): &(Token, usize, usize)) -> Result<Option<tree::Root>, String>
    {
        if token == &Token::Const {
            if let Some((tok, _, _)) = self.tokens.front() {
                if tok == &Token::Struct {
                    let st = self.parse_struct(*line, *col)?;
                    return Ok(Some(tree::Root::ConstantBuffer(st)));
                } else {
                    let prop = self.parse_property(*line, *col)?;
                    return Ok(Some(tree::Root::Constant(prop)));
                }
            }
            return Err(format!(
                "[Shader Annotation Language] Unexpected EOF at line {}, column {}",
                line, col
            ));
        }
        return Ok(None);
    }

    fn try_parse_vformat(&mut self, (token, line, col): &(Token, usize, usize)) -> Result<Option<tree::Root>, String>
    {
        if token == &Token::Vformat {
            let st = self.parse_struct(*line, *col)?;
            return Ok(Some(tree::Root::VertexFormat(st)));
        }
        return Ok(None);
    }

    fn parse_pipeline_val(&mut self, line: usize, col: usize) -> Result<tree::Value, String>
    {
        let (tok, line, col) = self.pop(line, col)?;
        if let Token::Float(f) = tok {
            return Ok(tree::Value::Float(f));
        } else if let Token::Int(i) = tok {
            return Ok(tree::Value::Int(i));
        } else if let Token::Bool(b) = tok {
            return Ok(tree::Value::Bool(b));
        } else if let Token::Identifier(id) = tok {
            return Ok(tree::Value::Identifier(id));
        }
        return Err(format!("[Shader Annotation Language] Unexpected token, expected identifier or namespace but got {} at line {}, column {}", &tok, line, col));
    }

    fn parse_var(&mut self, line: usize, col: usize) -> Result<tree::Variable, String>
    {
        let (tok, line, col) = self.pop(line, col)?;
        if let Token::Identifier(vname) = tok {
            let val = self.parse_pipeline_val(line, col)?;
            return Ok(tree::Variable {
                name: vname,
                value: val
            });
        }/* else if let Token::Namespace(vname) = tok {
            let val = self.parse_pipeline_val(line, col)?;
            return Ok(tree::Variable {
                name: vname,
                value: val
            });
        }*/
        return Err(format!("[Shader Annotation Language] Unexpected token, expected identifier or namespace but got {} at line {}, column {}", &tok, line, col));
    }

    fn parse_varlist(&mut self, line: usize, col: usize) -> Result<tree::VariableList, String>
    {
        let (tok, line, col) = self.pop(line, col)?;
        if let Token::Identifier(pname) = tok {
            let (tok, line, col) = self.pop(line, col)?;
            if tok == Token::BlockStart {
                let mut v = Vec::new();
                loop {
                    let var = self.parse_var(line, col)?;
                    v.push(var);
                    if self.check_block_end(line, col)? {
                        break;
                    }
                }
                return Ok(tree::VariableList {
                    name: pname,
                    variables: v
                });
            }
            return Err(format!(
                "[Shader Annotation Language] Unexpected token, expected '{{' but got {} at line {}, column {}",
                &tok, line, col
            ));
        }
        return Err(format!(
            "[Shader Annotation Language] Unexpected token, expected identifier but got {} at line {}, column {}",
            &tok, line, col
        ));
    }

    fn try_parse_pipeline(&mut self, (token, line, col): &(Token, usize, usize)) -> Result<Option<tree::Root>, String>
    {
        if token == &Token::Pipeline {
            let varlist = self.parse_varlist(*line, *col)?;
            return Ok(Some(tree::Root::Pipeline(varlist)));
        }
        return Ok(None);
    }

    fn try_parse_blendfunc(&mut self, (token, line, col): &(Token, usize, usize))
                           -> Result<Option<tree::Root>, String>
    {
        if token == &Token::Blendfunc {
            let varlist = self.parse_varlist(*line, *col)?;
            return Ok(Some(tree::Root::Blendfunc(varlist)));
        }
        return Ok(None);
    }

    pub fn parse(&mut self) -> Result<Vec<tree::Root>, String>
    {
        let mut dfj = Vec::new();

        while let Some(v) = self.tokens.pop_front() {
            if let Some(elem) = self.try_parse_use(&v)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_output(&v)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_vformat(&v)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_pipeline(&v)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_blendfunc(&v)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_const(&v)? {
                dfj.push(elem);
            } else {
                return Err(format!(
                    "[Shader Annotation Language] Unknown token at line {}, column {}",
                    v.1, v.2
                ));
            }
        }
        return Ok(dfj);
    }
}

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
use crate::lexer::{Lexer, TokenEntry};
use crate::lexer::token::Token;
use crate::parser::error::{Error, Type};
use crate::parser::tree;

use crate::lexer::token::Type as TokenType;

pub struct Parser
{
    tokens: VecDeque<TokenEntry>
}

impl Parser
{
    pub fn new(lexer: Lexer) -> Parser
    {
        return Parser {
            tokens: lexer.into_tokens()
        };
    }

    fn pop(&mut self, line: usize, col: usize) -> Result<TokenEntry, Error>
    {
        if let Some(entry) = self.tokens.pop_front() {
            Ok(entry)
        } else {
            Err(Error::new(line, col, Type::Eof))
        }
    }

    fn try_parse_use(&mut self, TokenEntry {token, line, col}: &TokenEntry) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Use {
            let TokenEntry { token, line, col } = self.pop(*line, *col)?;
            match token {
                Token::Identifier(module) => {
                    //pop_expect Colon
                    //pop_expect Colon
                    let TokenEntry { token, line, col } = self.pop(line, col)?;
                    match token {
                        Token::Identifier(member) => {
                            return Ok(Some(tree::Root::Use(tree::Use {
                                module,
                                member
                            })));
                        },
                        _ => return Err(Error::new(line, col, Type::UnexpectedToken {
                            expected: TokenType::Identifier,
                            actual: token
                        }))
                    };
                },
                _ => return Err(Error::new(line, col, Type::UnexpectedToken {
                    expected: TokenType::Identifier,
                    actual: token
                }))
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

    fn parse_property(&mut self, line: usize, col: usize) -> Result<tree::Property, Error>
    {
        let TokenEntry { token, line, col } = self.pop(line, col)?;
        if let Token::Identifier(t) = token {
            let TokenEntry { token, line, col } = self.pop(line, col)?;
            if let Token::Identifier(n) = token {
                return Ok(tree::Property { pname: n, ptype: t });
            }
            return Err(Error::new(line, col, Type::UnexpectedToken {
                expected: TokenType::Identifier,
                actual: token
            }));
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
        return Err(Error::new(line, col, Type::UnexpectedToken {
            expected: TokenType::Identifier,
            actual: token
        }));
    }

    fn try_parse_output(&mut self, TokenEntry {token, line, col}: &TokenEntry) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Output {
            let prop = self.parse_property(*line, *col)?;
            return Ok(Some(tree::Root::Output(prop)));
        }
        return Ok(None);
    }

    fn check_block_end(&mut self, line: usize, col: usize) -> Result<bool, Error>
    {
        if let Some(TokenEntry {token, ..}) = self.tokens.front() {
            if token == &Token::BlockEnd {
                self.pop(line, col)?;
                return Ok(true);
            }
        }
        return Ok(false);
    }

    fn parse_struct(&mut self, line: usize, col: usize) -> Result<tree::Struct, Error>
    {
        let TokenEntry { token, line, col } = self.pop(line, col)?;
        if token == Token::Struct {
            let TokenEntry { token, line, col } = self.pop(line, col)?;
            if let Token::Identifier(sname) = token {
                let TokenEntry { token, line, col } = self.pop(line, col)?;
                if token == Token::BlockStart {
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
                return Err(Error::new(line, col, Type::UnexpectedToken {
                    expected: TokenType::BlockStart,
                    actual: token
                }));
            }
            return Err(Error::new(line, col, Type::UnexpectedToken {
                expected: TokenType::Identifier,
                actual: token
            }));
        }
        return Err(Error::new(line, col, Type::UnexpectedToken {
            expected: TokenType::Struct,
            actual: token
        }));
    }

    fn try_parse_const(&mut self, TokenEntry {token, line, col}: &TokenEntry) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Const {
            if let Some(TokenEntry {token, ..}) = self.tokens.front() {
                if token == &Token::Struct {
                    let st = self.parse_struct(*line, *col)?;
                    return Ok(Some(tree::Root::ConstantBuffer(st)));
                } else {
                    let prop = self.parse_property(*line, *col)?;
                    return Ok(Some(tree::Root::Constant(prop)));
                }
            }
            return Err(Error::new(*line, *col, Type::Eof));
        }
        return Ok(None);
    }

    fn try_parse_vformat(&mut self, TokenEntry {token, line, col}: &TokenEntry) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Vformat {
            let st = self.parse_struct(*line, *col)?;
            return Ok(Some(tree::Root::VertexFormat(st)));
        }
        return Ok(None);
    }

    fn parse_pipeline_val(&mut self, line: usize, col: usize) -> Result<tree::Value, Error>
    {
        let TokenEntry { token, line, col } = self.pop(line, col)?;
        if let Token::Float(f) = token {
            return Ok(tree::Value::Float(f));
        } else if let Token::Int(i) = token {
            return Ok(tree::Value::Int(i));
        } else if let Token::Bool(b) = token {
            return Ok(tree::Value::Bool(b));
        } else if let Token::Identifier(id) = token {
            return Ok(tree::Value::Identifier(id));
        }
        return Err(Error::new(line, col, Type::UnexpectedToken {
            expected: TokenType::Literal,
            actual: token
        }));
    }

    fn parse_var(&mut self, line: usize, col: usize) -> Result<tree::Variable, Error>
    {
        let TokenEntry { token, line, col } = self.pop(line, col)?;
        if let Token::Identifier(vname) = token {
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
        return Err(Error::new(line, col, Type::UnexpectedToken {
            expected: TokenType::Identifier,
            actual: token
        }));
    }

    fn parse_varlist(&mut self, line: usize, col: usize) -> Result<tree::VariableList, Error>
    {
        let TokenEntry { token, line, col } = self.pop(line, col)?;
        if let Token::Identifier(pname) = token {
            let TokenEntry { token, line, col } = self.pop(line, col)?;
            if token == Token::BlockStart {
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
            return Err(Error::new(line, col, Type::UnexpectedToken {
                expected: TokenType::BlockStart,
                actual: token
            }));
        }
        return Err(Error::new(line, col, Type::UnexpectedToken {
            expected: TokenType::Identifier,
            actual: token
        }));
    }

    fn try_parse_pipeline(&mut self, TokenEntry {token, line, col}: &TokenEntry) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Pipeline {
            let varlist = self.parse_varlist(*line, *col)?;
            return Ok(Some(tree::Root::Pipeline(varlist)));
        }
        return Ok(None);
    }

    fn try_parse_blendfunc(&mut self, TokenEntry {token, line, col}: &TokenEntry)
                           -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Blendfunc {
            let varlist = self.parse_varlist(*line, *col)?;
            return Ok(Some(tree::Root::Blendfunc(varlist)));
        }
        return Ok(None);
    }

    pub fn parse(&mut self) -> Result<Vec<tree::Root>, Error>
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
                return Err(Error::new(v.line, v.col, Type::UnknownToken(v.token)));
            }
        }
        return Ok(dfj);
    }
}

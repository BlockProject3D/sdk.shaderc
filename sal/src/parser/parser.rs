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
    tokens: VecDeque<TokenEntry>,
    cur_line: usize,
    cur_column: usize
}

impl Parser
{
    pub fn new(lexer: Lexer) -> Parser
    {
        return Parser {
            tokens: lexer.into_tokens(),
            cur_line: 0,
            cur_column: 0
        };
    }

    fn pop_expect(&mut self, ttype: TokenType) -> Result<Token, Error>
    {
        let token = self.pop()?;
        if token.get_type() != ttype {
            Err(Error::new(self.cur_line, self.cur_column, Type::UnexpectedToken {
                expected: ttype,
                actual: token
            }))
        } else {
            Ok(token)
        }
    }

    fn pop(&mut self) -> Result<Token, Error>
    {
        if let Some(entry) = self.tokens.pop_front() {
            self.cur_column = entry.col;
            self.cur_line = entry.line;
            Ok(entry.token)
        } else {
            Err(Error::new(self.cur_line, self.cur_column, Type::Eof))
        }
    }

    fn try_parse_use(&mut self, token: &Token) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Use {
            let token = self.pop_expect(TokenType::Identifier)?;
            let module = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
            self.pop_expect(TokenType::Colon)?;
            self.pop_expect(TokenType::Colon)?;
            let token = self.pop_expect(TokenType::Identifier)?;
            let member = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
            Ok(Some(tree::Root::Use(tree::Use {
                module,
                member
            })))
        } else {
            Ok(None)
        }
    }

    fn parse_property(&mut self) -> Result<tree::Property, Error>
    {
        let token = self.pop_expect(TokenType::Identifier)?;
        let ptype = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
        let mut ptype_attr = None;
        let pname;
        let token = self.pop()?;
        match token {
            Token::Identifier(n) => pname = n,
            Token::Colon => {
                let token = self.pop_expect(TokenType::Identifier)?;
                ptype_attr = Some(token.identifier().unwrap()); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
                let token = self.pop_expect(TokenType::Identifier)?;
                pname = token.identifier().unwrap();
            },
            _ => return Err(Error::new(self.cur_line, self.cur_column, Type::UnexpectedToken {
                expected: TokenType::combined([TokenType::Identifier, TokenType::Colon]),
                actual: token
            }))
        };
        let token = self.pop()?;
        let pattr = match token {
            Token::Colon => {
                let token = self.pop_expect(TokenType::Identifier)?;
                self.pop_expect(TokenType::Break)?;
                Some(token.identifier().unwrap()) // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
            },
            Token::Break => None,
            _ => return Err(Error::new(self.cur_line, self.cur_column, Type::UnexpectedToken {
                expected: TokenType::combined([TokenType::Colon, TokenType::Break]),
                actual: token
            }))
        };
        Ok(tree::Property {
            pname,
            ptype,
            ptype_attr,
            pattr
        })
    }

    fn try_parse_output(&mut self, token: &Token) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Output {
            let prop = self.parse_property()?;
            return Ok(Some(tree::Root::Output(prop)));
        }
        return Ok(None);
    }

    fn check_block_end(&mut self) -> Result<bool, Error>
    {
        if let Some(TokenEntry {token, ..}) = self.tokens.front() {
            if token == &Token::BlockEnd {
                self.pop()?;
                return Ok(true);
            }
        }
        return Ok(false);
    }

    fn parse_struct(&mut self) -> Result<tree::Struct, Error>
    {
        self.pop_expect(TokenType::Struct)?;
        let token = self.pop_expect(TokenType::Identifier)?;
        let name = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
        self.pop_expect(TokenType::BlockStart)?;
        let mut props = Vec::new();
        loop {
            let prop = self.parse_property()?;
            props.push(prop);
            if self.check_block_end()? {
                break;
            }
        }
        Ok(tree::Struct {
            name,
            props
        })
    }

    fn try_parse_const(&mut self, token: &Token) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Const {
            if let Some(TokenEntry {token, ..}) = self.tokens.front() {
                if token == &Token::Struct {
                    let st = self.parse_struct()?;
                    return Ok(Some(tree::Root::ConstantBuffer(st)));
                } else {
                    let prop = self.parse_property()?;
                    return Ok(Some(tree::Root::Constant(prop)));
                }
            }
            return Err(Error::new(self.cur_line, self.cur_column, Type::Eof));
        }
        return Ok(None);
    }

    fn try_parse_vformat(&mut self, token: &Token) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Vformat {
            let st = self.parse_struct()?;
            return Ok(Some(tree::Root::VertexFormat(st)));
        }
        return Ok(None);
    }

    fn parse_pipeline_val(&mut self) -> Result<tree::Value, Error>
    {
        let token = self.pop()?;
        match token {
            Token::Float(f) => Ok(tree::Value::Float(f)),
            Token::Int(i) => Ok(tree::Value::Int(i)),
            Token::Bool(b) => Ok(tree::Value::Bool(b)),
            Token::Identifier(s) => Ok(tree::Value::Identifier(s)),
            _ => Err(Error::new(self.cur_line, self.cur_column, Type::UnexpectedToken {
                expected: TokenType::combined([TokenType::Float, TokenType::Int, TokenType::Bool, TokenType::Identifier]),
                actual: token
            }))
        }
    }

    fn parse_var(&mut self) -> Result<tree::Variable, Error>
    {
        let token = self.pop_expect(TokenType::Identifier)?;
        let name = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
        let token = self.pop()?;
        match token {
            Token::Eq => {
                let value = self.parse_pipeline_val()?;
                Ok(tree::Variable {
                    name,
                    value,
                    member: None
                })
            },
            Token::Colon => {
                self.pop_expect(TokenType::Colon)?;
                let token = self.pop_expect(TokenType::Identifier)?;
                let member = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
                self.pop_expect(TokenType::Eq)?;
                let value = self.parse_pipeline_val()?;
                Ok(tree::Variable {
                    name,
                    value,
                    member: Some(member)
                })
            },
            _ => Err(Error::new(self.cur_line, self.cur_column, Type::UnexpectedToken {
                expected: TokenType::combined([TokenType::Eq, TokenType::Colon]),
                actual: token
            }))
        }
    }

    fn parse_varlist(&mut self) -> Result<tree::VariableList, Error>
    {
        let token = self.pop_expect(TokenType::Identifier)?;
        let name = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
        self.pop_expect(TokenType::BlockStart)?;
        let mut vars = Vec::new();
        loop {
            let var = self.parse_var()?;
            vars.push(var);
            if self.check_block_end()? {
                break;
            }
        }
        Ok(tree::VariableList {
            name,
            vars
        })
    }

    fn try_parse_pipeline(&mut self, token: &Token) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Pipeline {
            let varlist = self.parse_varlist()?;
            return Ok(Some(tree::Root::Pipeline(varlist)));
        }
        return Ok(None);
    }

    fn try_parse_blendfunc(&mut self, token: &Token) -> Result<Option<tree::Root>, Error>
    {
        if token == &Token::Blendfunc {
            let varlist = self.parse_varlist()?;
            return Ok(Some(tree::Root::Blendfunc(varlist)));
        }
        return Ok(None);
    }

    pub fn parse(&mut self) -> Result<Vec<tree::Root>, Error>
    {
        let mut dfj = Vec::new();

        while let Some(v) = self.tokens.pop_front() {
            if let Some(elem) = self.try_parse_use(&v.token)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_output(&v.token)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_vformat(&v.token)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_pipeline(&v.token)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_blendfunc(&v.token)? {
                dfj.push(elem);
            } else if let Some(elem) = self.try_parse_const(&v.token)? {
                dfj.push(elem);
            } else {
                return Err(Error::new(v.line, v.col, Type::UnknownToken(v.token)));
            }
        }
        return Ok(dfj);
    }
}

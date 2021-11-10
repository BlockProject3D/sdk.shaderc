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

    fn pop_expect(&mut self, ttype: TokenType, line: usize, col: usize) -> Result<TokenEntry, Error>
    {
        let entry = self.pop(line, col)?;
        if entry.token.get_type() != ttype {
            Err(Error::new(line, col, Type::UnexpectedToken {
                expected: ttype,
                actual: entry.token
            }))
        } else {
            Ok(entry)
        }
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
            let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, *line, *col)?;
            let module = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
            let TokenEntry { line, col, .. } = self.pop_expect(TokenType::Colon, line, col)?;
            let TokenEntry { line, col, .. } = self.pop_expect(TokenType::Colon, line, col)?;
            let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, line, col)?;
            let member = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
            Ok(Some(tree::Root::Use(tree::Use {
                module,
                member
            })))
        } else {
            Ok(None)
        }
    }

    fn parse_property(&mut self, line: usize, col: usize) -> Result<tree::Property, Error>
    {
        let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, line, col)?;
        let ptype = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
        let mut ptype_attr = None;
        let pname;
        let l;
        let c;
        let TokenEntry { token, line, col } = self.pop(line, col)?;
        match token {
            Token::Identifier(n) => {
                pname = n;
                l = line;
                c = col;
            },
            Token::Colon => {
                let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, line, col)?;
                ptype_attr = Some(token.identifier().unwrap()); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
                let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, line, col)?;
                pname = token.identifier().unwrap();
                l = line;
                c = col;
            },
            _ => return Err(Error::new(line, col, Type::UnexpectedToken {
                expected: TokenType::Identifier,
                actual: token
            }))
        };
        let TokenEntry { token, line, col } = self.pop(l, c)?;
        let pattr = match token {
            Token::Colon => {
                let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, line, col)?;
                self.pop_expect(TokenType::Break, line, col)?;
                Some(token.identifier().unwrap()) // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
            },
            Token::Break => None,
            _ => return Err(Error::new(line, col, Type::UnexpectedToken {
                expected: TokenType::Break,
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
        let TokenEntry { line, col, .. } = self.pop_expect(TokenType::Struct, line, col)?;
        let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, line, col)?;
        let name = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
        let TokenEntry { line, col, .. } = self.pop_expect(TokenType::BlockStart, line, col)?;
        let mut props = Vec::new();
        loop {
            let prop = self.parse_property(line, col)?;
            props.push(prop);
            if self.check_block_end(line, col)? {
                break;
            }
        }
        Ok(tree::Struct {
            name,
            props
        })
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
        match token {
            Token::Float(f) => Ok(tree::Value::Float(f)),
            Token::Int(i) => Ok(tree::Value::Int(i)),
            Token::Bool(b) => Ok(tree::Value::Bool(b)),
            Token::Identifier(s) => Ok(tree::Value::Identifier(s)),
            _ => Err(Error::new(line, col, Type::UnexpectedToken {
                expected: TokenType::Literal,
                actual: token
            }))
        }
    }

    fn parse_var(&mut self, line: usize, col: usize) -> Result<tree::Variable, Error>
    {
        let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, line, col)?;
        let name = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
        let TokenEntry { token, line, col } = self.pop(line, col)?;
        match token {
            Token::Eq => {
                let value = self.parse_pipeline_val(line, col)?;
                Ok(tree::Variable {
                    name,
                    value,
                    member: None
                })
            },
            Token::Colon => {
                let TokenEntry { line, col, .. } = self.pop_expect(TokenType::Colon, line, col)?;
                let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, line, col)?;
                let member = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
                let TokenEntry { line, col, .. } = self.pop_expect(TokenType::Eq, line, col)?;
                let value = self.parse_pipeline_val(line, col)?;
                Ok(tree::Variable {
                    name,
                    value,
                    member: Some(member)
                })
            },
            _ => Err(Error::new(line, col, Type::UnexpectedToken {
                expected: TokenType::Eq,
                actual: token
            }))
        }
    }

    fn parse_varlist(&mut self, line: usize, col: usize) -> Result<tree::VariableList, Error>
    {
        let TokenEntry { token, line, col } = self.pop_expect(TokenType::Identifier, line, col)?;
        let name = token.identifier().unwrap(); // SAFETY: we have tested for identifier in pop_expect so no panic possible here!
        let TokenEntry { line, col, .. } = self.pop_expect(TokenType::BlockStart, line, col)?;
        let mut vars = Vec::new();
        loop {
            let var = self.parse_var(line, col)?;
            vars.push(var);
            if self.check_block_end(line, col)? {
                break;
            }
        }
        Ok(tree::VariableList {
            name,
            vars
        })
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

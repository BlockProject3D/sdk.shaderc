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

use crate::sal::lexer::Lexer;
use crate::sal::lexer::Token;

pub mod tree
{
    use crate::sal::ast::PipelineStatement;

    pub struct Property
    {
        pub ptype: String,
        pub pname: String    
    }

    pub struct Struct
    {
        pub name: String,
        pub properties: Vec<Property>
    }

    pub struct Use
    {
        pub module: String,
        pub member: String
    }

    pub enum Root
    {
        Constant(Property),
        ConstantBuffer(Struct),
        Output(Property),
        VertexFormat(Struct),
        Use(Use),
        Pipeline(PipelineStatement)
    }
}

pub struct Parser
{
    tokens: VecDeque<(Token, usize, usize)>
}

impl Parser
{
    fn new(lexer: Lexer) -> Parser
    {
        return Parser
        {
            tokens: lexer.get_tokens()
        };
    }

    fn pop(&mut self, line: usize, col: usize) -> Result<(Token, usize, usize), String>
    {
        if let Some((tok, line, col)) = self.tokens.pop_front()
        {
            return Ok((tok, line, col));
        }
        return Err(format!("[Shader Annotation Language] Unexpected EOF at line {}, column {}", line, col));
    }

    fn try_parse_use(&mut self, (token, line, col): &(Token, usize, usize)) -> Result<(), String>
    {
        if token == &Token::Use
        {
            let (tok, line, col) = self.pop(*line, *col)?;
            if let Token::Namespace(n) = tok
            {
                let v: Vec<&str> = n.split("::").collect();
                if v.len() != 2
                {
                    return Err(format!("[Shader Annotation Language] Bad namespace path format ('{}') at line {}, column {}", &n, line, col))
                }
                let module = String::from(v[0]);
                let member = String::from(v[0]);
                return Ok(());
            }
            return Err(format!("[Shader Annotation Language] Unexpected token, expected namespace but got {} at line {}, column {}", &tok, line, col));
        }
        return Ok(());
    }

    fn parse(&mut self)
    {
    }
}

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

use std::fmt::{Debug, Display, Formatter};

use crate::{
    lexer::Lexer,
    parser::Parser
};
use crate::ast::{AstBuilder, RefResolver, Visitor};
use crate::parser::error::ParserOrVisitor;

#[derive(Debug)]
pub enum AutoError<T, E>
{
    Lexer(crate::lexer::error::Error),
    Parser(crate::parser::error::Error),
    Ast(crate::ast::error::Error<T, E>)
}

impl<T: Display, E: Debug> Display for AutoError<T, E>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            AutoError::Lexer(e) => write!(f, "lexer error: {}", e),
            AutoError::Parser(e) => write!(f, "parser error: {}", e),
            AutoError::Ast(e) => write!(f, "ast generation error: {}", e)
        }
    }
}

impl<T, E> From<crate::parser::error::ParserOrVisitor<crate::ast::error::Error<T, E>>> for AutoError<T, E>
{
    fn from(e: ParserOrVisitor<crate::ast::error::Error<T, E>>) -> Self {
        match e {
            ParserOrVisitor::Visitor(e) => AutoError::Ast(e),
            ParserOrVisitor::Parser(e) => AutoError::Parser(e)
        }
    }
}

pub fn auto_lexer_parser<T: AsRef<[u8]>, A: RefResolver, V: Visitor<A>>(
    buf: T,
    ast: A,
    visitor: V
) -> Result<A, AutoError<A::Key, V::Error>>
{
    let mut lexer = Lexer::new();
    lexer.process(buf.as_ref()).map_err(AutoError::Lexer)?;
    auto_parser(lexer, ast, visitor)
}

pub fn auto_parser<A: RefResolver, V: Visitor<A>>(
    lexer: Lexer,
    ast: A,
    visitor: V
) -> Result<A, AutoError<A::Key, V::Error>>
{
    let mut parser = Parser::new(lexer);
    let ast = parser.parse(AstBuilder::new(ast, visitor))?.into_inner();
    Ok(ast)
}

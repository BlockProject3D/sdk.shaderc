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

use std::fmt::Debug;
use crate::ast::{build_ast, UseResolver};
use crate::ast::tree::Statement;
use crate::Lexer;
use crate::Parser;

pub enum AutoError<ResolverError: Debug>
{
    Lexer(crate::lexer::error::Error),
    Parser(crate::parser::error::Error),
    Ast(crate::ast::error::Error<ResolverError>)
}

impl<ResolverError: Debug> From<crate::lexer::error::Error> for AutoError<ResolverError>
{
    fn from(e: crate::lexer::error::Error) -> Self
    {
        Self::Lexer(e)
    }
}

impl<ResolverError: Debug> From<crate::parser::error::Error> for AutoError<ResolverError>
{
    fn from(e: crate::parser::error::Error) -> Self
    {
        Self::Parser(e)
    }
}

impl<ResolverError: Debug> From<crate::ast::error::Error<ResolverError>> for AutoError<ResolverError>
{
    fn from(e: crate::ast::error::Error<ResolverError>) -> Self
    {
        Self::Ast(e)
    }
}

pub fn auto_lexer_parser<T: AsRef<[u8]>, Resolver: UseResolver>(buf: T, resolver: Resolver) -> Result<Vec<Statement>, AutoError<Resolver::Error>>
{
    let mut lexer = Lexer::new();
    lexer.process(buf.as_ref())?;
    auto_parser(lexer, resolver)
}

pub fn auto_parser<Resolver: UseResolver>(lexer: Lexer, resolver: Resolver) -> Result<Vec<Statement>, AutoError<Resolver::Error>>
{
    let mut parser = Parser::new(lexer);
    let roots = parser.parse()?;
    let ast = build_ast(roots, resolver)?;
    Ok(ast)
}

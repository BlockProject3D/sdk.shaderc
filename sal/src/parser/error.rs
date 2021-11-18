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

/*#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnexpectedToken
{
    pub actual: Token,
    pub expected: Token
}*/

use std::fmt::{Display, Formatter, write};
use crate::lexer::token::{Token, Type as TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum Type
{
    UnexpectedToken
    {
        actual: Token,
        expected: TokenType
    },
    UnknownToken(Token),
    Eof
}

impl Display for Type
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Type::UnexpectedToken { actual, expected } => write!(f, "unexpected token (expected {}, got {})", expected, actual),
            Type::UnknownToken(token) => write!(f, "unknown token ({})", token),
            Type::Eof => f.write_str("unexpected EOF")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error
{
    pub line: usize,
    pub col: usize,
    pub etype: Type
}

impl Error
{
    pub fn new(line: usize, col: usize, etype: Type) -> Self
    {
        Self { line, col, etype }
    }
}

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}:{} {}", self.line, self.col, self.etype)
    }
}

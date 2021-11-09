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

use std::fmt::{Display, Write};
use std::str::from_utf8_unchecked;

pub const STR_CONST: &'static [u8] = b"const";
pub const STR_STRUCT: &'static [u8] = b"struct";
pub const STR_PIPELINE: &'static [u8] = b"pipeline";
pub const STR_BLENDFUNC: &'static [u8] = b"blendfunc";
pub const STR_VFORMAT: &'static [u8] = b"vformat";
pub const STR_USE: &'static [u8] = b"use";
pub const STR_OUTPUT: &'static [u8] = b"output";
pub const STR_TRUE: &'static [u8] = b"true";
pub const STR_FALSE: &'static [u8] = b"false";

pub const CHR_BREAK: u8 = b';';
pub const CHR_EQ: u8 = b'=';
pub const CHR_BLOCK_START: u8 = b'{';
pub const CHR_BLOCK_END: u8 = b'}';
pub const CHR_COMMENT: u8 = b'#';
pub const CHR_COLON: u8 = b':';

pub const CHR_NL: u8 = b'\n';

#[derive(Clone, Debug, PartialEq)]
pub enum Token
{
    Const,
    Struct,
    Pipeline,
    Vformat,
    Use,
    Eq,
    BlockStart,
    BlockEnd,
    Output,
    Bool(bool),
    Int(i32),
    Float(f32),
    Identifier(String),
    Colon,
    Blendfunc,
    Whitespace,
    Break
}

impl Display for Token
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        unsafe {
            //SAFETY: we know that token litterals are valid UTF8 as they are valid ASCII
            match self {
                Token::Const => formatter.write_str(from_utf8_unchecked(STR_CONST)),
                Token::Struct => formatter.write_str(from_utf8_unchecked(STR_STRUCT)),
                Token::Pipeline => formatter.write_str(from_utf8_unchecked(STR_PIPELINE)),
                Token::Vformat => formatter.write_str(from_utf8_unchecked(STR_VFORMAT)),
                Token::Use => formatter.write_str(from_utf8_unchecked(STR_USE)),
                Token::Eq => formatter.write_char(CHR_EQ as char),
                Token::BlockStart => formatter.write_char(CHR_BLOCK_START as char),
                Token::BlockEnd => formatter.write_char(CHR_BLOCK_END as char),
                Token::Output => formatter.write_str(from_utf8_unchecked(STR_OUTPUT)),
                Token::Bool(b) => write!(formatter, "{}", b),
                Token::Int(i) => write!(formatter, "{}", i),
                Token::Float(f) => write!(formatter, "{}", f),
                Token::Identifier(s) => formatter.write_str(s),
                Token::Colon => formatter.write_char(CHR_COLON as char),
                Token::Blendfunc => formatter.write_str(from_utf8_unchecked(STR_BLENDFUNC)),
                Token::Whitespace => formatter.write_str("whitespace"),
                Token::Break => formatter.write_char(CHR_BREAK as char)
            }
        }
    }
}

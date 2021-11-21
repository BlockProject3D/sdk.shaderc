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

use std::fmt::{Display, Formatter};

pub const STR_CONST: &[u8] = b"const";
pub const STR_STRUCT: &[u8] = b"struct";
pub const STR_PIPELINE: &[u8] = b"pipeline";
pub const STR_BLENDFUNC: &[u8] = b"blendfunc";
pub const STR_VFORMAT: &[u8] = b"vformat";
pub const STR_USE: &[u8] = b"use";
pub const STR_OUTPUT: &[u8] = b"output";
pub const STR_TRUE: &[u8] = b"true";
pub const STR_FALSE: &[u8] = b"false";

pub const CHR_BREAK: u8 = b';';
pub const CHR_EQ: u8 = b'=';
pub const CHR_BLOCK_START: u8 = b'{';
pub const CHR_BLOCK_END: u8 = b'}';
pub const CHR_COMMENT: u8 = b'#';
pub const CHR_COLON: u8 = b':';
pub const CHR_ARRAY_START: u8 = b'[';
pub const CHR_ARRAY_END: u8 = b']';

pub const CHR_NL: u8 = b'\n';

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type
{
    Const,
    Struct,
    Pipeline,
    Vformat,
    Use,
    Eq,
    BlockStart,
    BlockEnd,
    ArrayStart,
    ArrayEnd,
    Output,
    Bool,
    Int,
    Float,
    Identifier,
    Colon,
    Blendfunc,
    Whitespace,
    Break,
    Combined(Vec<Type>)
}

impl Type
{
    pub fn name(&self) -> &'static str
    {
        match self {
            Type::Const => "const",
            Type::Struct => "struct",
            Type::Pipeline => "pipeline",
            Type::Vformat => "vformat",
            Type::Use => "use",
            Type::Eq => "'='",
            Type::BlockStart => "'{'",
            Type::BlockEnd => "'}'",
            Type::Output => "output",
            Type::Bool => "bool",
            Type::Int => "int",
            Type::Float => "float",
            Type::Identifier => "identifier",
            Type::Colon => "':'",
            Type::Blendfunc => "blendfunc",
            Type::Whitespace => "whitespace",
            Type::Break => "';'",
            Type::Combined(_) => "combined",
            Type::ArrayStart => "'['",
            Type::ArrayEnd => "']'"
        }
    }

    pub fn combined<T: AsRef<[Type]>>(t: T) -> Self
    {
        Self::Combined(t.as_ref().into())
    }
}

impl Display for Type
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        if let Type::Combined(v) = self {
            for (i, t) in v.iter().enumerate() {
                f.write_str(t.name())?;
                if i != v.len() - 1 {
                    f.write_str(" or ")?;
                }
            }
            Ok(())
        } else {
            f.write_str(self.name())
        }
    }
}

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
    ArrayStart,
    ArrayEnd,
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

impl Token
{
    pub fn get_type(&self) -> Type
    {
        match self {
            Token::Const => Type::Const,
            Token::Struct => Type::Struct,
            Token::Pipeline => Type::Pipeline,
            Token::Vformat => Type::Vformat,
            Token::Use => Type::Use,
            Token::Eq => Type::Eq,
            Token::BlockStart => Type::BlockStart,
            Token::BlockEnd => Type::BlockEnd,
            Token::ArrayStart => Type::ArrayStart,
            Token::ArrayEnd => Type::ArrayEnd,
            Token::Output => Type::Output,
            Token::Bool(_) => Type::Bool,
            Token::Int(_) => Type::Int,
            Token::Float(_) => Type::Float,
            Token::Identifier(_) => Type::Identifier,
            Token::Colon => Type::Colon,
            Token::Blendfunc => Type::Blendfunc,
            Token::Whitespace => Type::Whitespace,
            Token::Break => Type::Break
        }
    }

    pub fn identifier(self) -> Option<String>
    {
        if let Token::Identifier(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn bool(self) -> Option<bool>
    {
        if let Token::Bool(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn int(self) -> Option<i32>
    {
        if let Token::Int(i) = self {
            Some(i)
        } else {
            None
        }
    }

    pub fn float(self) -> Option<f32>
    {
        if let Token::Float(f) = self {
            Some(f)
        } else {
            None
        }
    }
}

impl Display for Token
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        match self {
            Token::Bool(b) => write!(formatter, "bool({})", b),
            Token::Int(i) => write!(formatter, "int({})", i),
            Token::Float(f) => write!(formatter, "float({})", f),
            Token::Identifier(s) => write!(formatter, "identifier({})", s),
            _ => formatter.write_str(self.get_type().name())
        }
    }
}

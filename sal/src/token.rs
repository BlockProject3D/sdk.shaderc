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

use std::fmt::Display;

pub const STR_BREAK: &'static str = ";";
pub const STR_CONST: &'static str = "const";
pub const STR_STRUCT: &'static str = "struct";
pub const STR_PIPELINE: &'static str = "pipeline";
pub const STR_BLENDFUNC: &'static str = "blendfunc";
pub const STR_VFORMAT: &'static str = "vformat";
pub const STR_USE: &'static str = "use";
pub const STR_EQ: &'static str = "=";
pub const STR_BLOCK_START: &'static str = "{";
pub const STR_BLOCK_END: &'static str = "}";
pub const STR_COMMENT: &'static str = "#";
pub const STR_OUTPUT: &'static str = "output";

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
    Namespace(String),
    Blendfunc
}

impl Display for Token
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        match self {
            Token::Const => formatter.write_str(STR_CONST)?,
            Token::Struct => formatter.write_str(STR_STRUCT)?,
            Token::Pipeline => formatter.write_str(STR_PIPELINE)?,
            Token::Vformat => formatter.write_str(STR_VFORMAT)?,
            Token::Use => formatter.write_str(STR_USE)?,
            Token::Eq => formatter.write_str("'='")?,
            Token::BlockStart => formatter.write_str("'{'")?,
            Token::BlockEnd => formatter.write_str("'}'")?,
            Token::Output => formatter.write_str(STR_OUTPUT)?,
            Token::Bool(_) => formatter.write_str("bool")?,
            Token::Int(_) => formatter.write_str("int")?,
            Token::Float(_) => formatter.write_str("float")?,
            Token::Identifier(_) => formatter.write_str("identifier")?,
            Token::Namespace(_) => formatter.write_str("namespace")?,
            Token::Blendfunc => formatter.write_str(STR_BLENDFUNC)?
        }
        return Ok(());
    }
}

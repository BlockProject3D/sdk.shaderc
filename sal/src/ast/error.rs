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

use std::{fmt::Debug, num::ParseIntError};

use crate::{ast::tree as ast, parser::tree};

#[derive(Clone, Debug)]
pub enum ValueType
{
    Bool,
    Float,
    Int,
    Enum,
    Identifier
}

#[derive(Clone, Debug)]
pub enum TypeError
{
    VectorSize(ParseIntError),
    UnknownVector(String),
    UnknownTexture(String),
    Unknown(String),
    Banned(ast::PropertyType)
}

#[derive(Clone, Debug)]
pub enum ValueError
{
    UnknownEnum(String),
    UnknownVariable(String),
    Unexpected
    {
        expected: ValueType,
        actual: tree::Value
    }
}

#[derive(Clone, Debug)]
pub enum Error<ResolverError: Debug>
{
    Type(TypeError),
    Value(ValueError),
    UnresolvedUse(ResolverError)
}

impl<ResolverError: Debug> From<TypeError> for Error<ResolverError>
{
    fn from(e: TypeError) -> Self
    {
        Self::Type(e)
    }
}

impl<ResolverError: Debug> From<ValueError> for Error<ResolverError>
{
    fn from(e: ValueError) -> Self
    {
        Self::Value(e)
    }
}

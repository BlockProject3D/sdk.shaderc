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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property
{
    pub ptype: String,
    pub ptype_attr: Option<String>,
    pub pname: String,
    pub pattr: Option<String>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct
{
    pub name: String,
    pub props: Vec<Property>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use
{
    pub module: String,
    pub member: String
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value
{
    Int(i32),
    Float(f32),
    Bool(bool),
    Identifier(String)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable
{
    pub name: String,
    pub member: Option<String>,
    pub value: Value
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableList
{
    pub name: String,
    pub vars: Vec<Variable>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Root
{
    Constant(Property),
    ConstantBuffer(Struct),
    Output(Property),
    VertexFormat(Struct),
    Use(Use),
    Pipeline(VariableList),
    Blendfunc(VariableList)
}

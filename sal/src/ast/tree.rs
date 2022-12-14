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
use serde::{Serialize, Deserialize};

pub trait VarlistStatement
{
    fn new(name: String) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum BaseType
{
    Int,
    Float,
    Uint,
    Bool,
    Double
}

impl BaseType
{
    pub fn get_name(&self) -> &'static str
    {
        match self {
            BaseType::Int => "int",
            BaseType::Float => "float",
            BaseType::Uint => "uint",
            BaseType::Bool => "bool",
            BaseType::Double => "double"
        }
    }

    pub fn get_char(&self) -> char
    {
        match self {
            BaseType::Int => 'i',
            BaseType::Float => 'f',
            BaseType::Uint => 'u',
            BaseType::Bool => 'b',
            BaseType::Double => 'd'
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct VectorType
{
    pub item: BaseType,
    pub size: u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TextureType
{
    Scalar(BaseType),
    Vector(VectorType)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayItemType<T>
{
    Vector(VectorType),
    Matrix(VectorType),
    StructRef(T)
}

impl<T: Copy> Copy for ArrayItemType<T> {}

impl<T: Display> Display for ArrayItemType<T>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            ArrayItemType::Vector(v) => write!(f, "vec{}{}", v.size, v.item.get_char()),
            ArrayItemType::Matrix(m) => write!(f, "mat{}{}", m.size, m.item.get_char()),
            ArrayItemType::StructRef(s) => write!(f, "StructRef({})", s)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayType<T>
{
    pub size: u32,
    pub item: ArrayItemType<T>
}

impl<T: Copy> Copy for ArrayType<T> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyType<T>
{
    Scalar(BaseType),
    Vector(VectorType),
    Matrix(VectorType),
    Sampler,
    Texture2D(TextureType),
    Texture3D(TextureType),
    Texture2DArray(TextureType),
    TextureCube(TextureType),
    StructRef(T),
    Array(ArrayType<T>)
}

impl<T: Copy> Copy for PropertyType<T> {}

impl<T: Display> Display for PropertyType<T>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        let mut fmt_texture_type = |name: &'static str, t: &TextureType| {
            match t {
                TextureType::Scalar(s) => write!(f, "{}<{}>", name, s.get_name()),
                TextureType::Vector(v) => write!(f, "{}<vec{}{}>", name, v.size, v.item.get_char())
            }
        };
        match self {
            PropertyType::Scalar(s) => write!(f, "{}", s.get_name()),
            PropertyType::Vector(v) => write!(f, "vec{}{}", v.size, v.item.get_char()),
            PropertyType::Matrix(m) => write!(f, "mat{}{}", m.size, m.item.get_char()),
            PropertyType::Sampler => f.write_str("Sampler"),
            PropertyType::Texture2D(t) => fmt_texture_type("Texture2D", t),
            PropertyType::Texture3D(t) => fmt_texture_type("Texture3D", t),
            PropertyType::Texture2DArray(t) => fmt_texture_type("Texture2DArray", t),
            PropertyType::TextureCube(t) => fmt_texture_type("TextureCube", t),
            PropertyType::StructRef(s) => write!(f, "StructRef({})", s),
            PropertyType::Array(a) => write!(f, "{}[{}]", a.item, a.size)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribute
{
    Identifier(String),
    Order(u32),
    Pack
}

impl Attribute
{
    pub fn get_order(&self) -> Option<u32>
    {
        match self {
            Attribute::Identifier(_) => None,
            Attribute::Order(o) => Some(*o),
            Attribute::Pack => None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property<T = String>
{
    pub ptype: PropertyType<T>,
    pub pname: String,
    pub pattr: Option<Attribute>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct<T = String>
{
    pub name: String,
    pub attr: Option<Attribute>,
    pub props: Vec<Property<T>>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderMode
{
    Triangles,
    Wireframe,
    Patches
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CullingMode
{
    BackFace,
    FrontFace,
    Disabled
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStatement
{
    pub name: String,
    pub depth_enable: bool,
    pub depth_write_enable: bool,
    pub scissor_enable: bool,
    pub render_mode: RenderMode,
    pub culling_mode: CullingMode
}

impl VarlistStatement for PipelineStatement
{
    fn new(name: String) -> Self
    {
        PipelineStatement {
            name,
            depth_enable: true,
            depth_write_enable: true,
            scissor_enable: false,
            render_mode: RenderMode::Triangles,
            culling_mode: CullingMode::BackFace
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendFactor
{
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstColor,
    OneMinusDstColor,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturate,
    Src1Color,
    OneMinusSrc1Color,
    Src1Alpha,
    OneMinusSrc1Alpha
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendOperator
{
    Add,
    Subtract,
    InverseSubtract,
    Min,
    Max
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlendfuncStatement
{
    pub name: String,
    pub src_color: BlendFactor,
    pub dst_color: BlendFactor,
    pub src_alpha: BlendFactor,
    pub dst_alpha: BlendFactor,
    pub color_op: BlendOperator,
    pub alpha_op: BlendOperator
}

impl VarlistStatement for BlendfuncStatement
{
    fn new(name: String) -> Self
    {
        BlendfuncStatement {
            name,
            src_color: BlendFactor::One,
            dst_color: BlendFactor::Zero,
            src_alpha: BlendFactor::One,
            dst_alpha: BlendFactor::Zero,
            color_op: BlendOperator::Add,
            alpha_op: BlendOperator::Add
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<T = String>
{
    Constant(Property<T>),
    ConstantBuffer(Struct<T>),
    Output(Property<T>),
    VertexFormat(Struct<T>),
    Pipeline(PipelineStatement),
    Blendfunc(BlendfuncStatement),
    Noop // Used to represent a statement to ignore in the parse tree
}

impl<T> Statement<T>
{
    pub fn get_name(&self) -> Option<&str>
    {
        match self {
            Statement::Constant(v) => Some(&v.pname),
            Statement::ConstantBuffer(v) => Some(&v.name),
            Statement::Output(v) => Some(&v.pname),
            Statement::VertexFormat(v) => Some(&v.name),
            Statement::Pipeline(v) => Some(&v.name),
            Statement::Blendfunc(v) => Some(&v.name),
            Statement::Noop => None
        }
    }
}

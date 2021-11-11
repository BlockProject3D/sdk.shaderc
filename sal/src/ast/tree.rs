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

use std::collections::HashMap;

pub trait VarlistStatement
{
    fn new(name: String) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType
{
    Int,
    Float,
    Uint,
    Bool,
    Double
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VectorType
{
    pub item: BaseType,
    pub size: u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureType
{
    Scalar(BaseType),
    Vector(VectorType)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType
{
    Scalar(BaseType),
    Vector(VectorType),
    Matrix(VectorType),
    Sampler,
    Texture2D(TextureType),
    Texture3D(TextureType),
    Texture2DArray(TextureType),
    TextureCube(TextureType)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property
{
    pub ptype: PropertyType,
    pub pname: String,
    pub pattr: Option<String>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct
{
    pub name: String,
    pub props: Vec<Property>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode
{
    Triangles,
    Wireframe,
    Patches
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        return PipelineStatement {
            name,
            depth_enable: true,
            depth_write_enable: true,
            scissor_enable: false,
            render_mode: RenderMode::Triangles,
            culling_mode: CullingMode::BackFace
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        return BlendfuncStatement {
            name,
            src_color: BlendFactor::One,
            dst_color: BlendFactor::Zero,
            src_alpha: BlendFactor::One,
            dst_alpha: BlendFactor::Zero,
            color_op: BlendOperator::Add,
            alpha_op: BlendOperator::Add
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement
{
    Constant(Property),
    ConstantBuffer(Struct),
    Output(Property),
    VertexFormat(Struct),
    Pipeline(PipelineStatement),
    Blendfunc(BlendfuncStatement),
    Noop // Used to represent a statement to ignore in the parse tree
}

impl Statement
{
    pub fn get_name(&self) -> Option<&str>
    {
        return match self {
            Statement::Constant(v) => Some(&v.pname),
            Statement::ConstantBuffer(v) => Some(&v.name),
            Statement::Output(v) => Some(&v.pname),
            Statement::VertexFormat(v) => Some(&v.name),
            Statement::Pipeline(v) => Some(&v.name),
            Statement::Blendfunc(v) => Some(&v.name),
            Statement::Noop => None
        };
    }
}

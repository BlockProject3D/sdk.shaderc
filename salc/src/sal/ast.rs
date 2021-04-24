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
use std::vec::Vec;
use std::string::String;

pub enum BaseType
{
    Int,
    Float,
    Uint,
    Bool,
    Double
}

pub struct PropertyType
{
    base_type: BaseType,
    count: u8
}

pub struct Property
{
    pub ptype: PropertyType,
    pub pname: String
}

pub struct Struct
{
    pub name: String,
    pub properties: Vec<Property>
}

pub enum Value
{
    Int(i32),
    Float(f32),
    Bool(bool),
    Enum(String)
}

pub enum RenderMode
{
    Triangles,
    Wireframe,
    Patches
}

pub enum CullingMode
{
    BackFace,
    FrontFace,
    Disabled
}

pub struct PipelineStatement
{
    pub name: String,
    pub depth_enable: bool,
    pub depth_write_enable: bool,
    pub scissor_enable: bool,
    pub render_mode: RenderMode,
    pub culling_mode: CullingMode,
    pub blend_state_factor: (f32, f32, f32, f32),
    pub blend_functions: HashMap<String, String>
}

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

pub enum BlendOperator
{
    Add,
    Subtract,
    InverseSubtract,
    Min,
    Max
}

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

pub enum Statement
{
    Constant(Property),
    ConstantBuffer(Struct),
    Output(Property),
    VertexFormat(Struct),
    Pipeline(PipelineStatement),
    Blendfunc(BlendfuncStatement)
}

pub enum Root
{
    Use(Statement),
    Statement(Statement)
}

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

pub trait VarlistStatement
{
    fn new(name: String) -> Self;
}

#[derive(Clone, Copy)]
pub enum BaseType
{
    Int,
    Float,
    Uint,
    Bool,
    Double
}

#[derive(Clone, Copy)]
pub enum PropertyType
{
    Scalar(BaseType),
    Vector(BaseType, u8),
    Matrix(BaseType, u8),
    Sampler,
    Texture2D(BaseType, u8),
    Texture3D(BaseType, u8),
    Texture2DArray(BaseType, u8),
    TextureCube(BaseType, u8)
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

#[derive(Clone, Copy)]
pub enum RenderMode
{
    Triangles,
    Wireframe,
    Patches
}

#[derive(Clone, Copy)]
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
    pub blend_functions: HashMap<String, String>
}

impl VarlistStatement for PipelineStatement
{
    fn new(name: String) -> Self
    {
        return PipelineStatement
        {
            name: name,
            depth_enable: true,
            depth_write_enable: true,
            scissor_enable: false,
            render_mode: RenderMode::Triangles,
            culling_mode: CullingMode::BackFace,
            blend_functions: HashMap::new()
        }
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

impl VarlistStatement for BlendfuncStatement
{
    fn new(name: String) -> Self
    {
        return BlendfuncStatement
        {
            name: name,
            src_color: BlendFactor::One,
            dst_color: BlendFactor::Zero,
            src_alpha: BlendFactor::One,
            dst_alpha: BlendFactor::Zero,
            color_op: BlendOperator::Add,
            alpha_op: BlendOperator::Add
        }
    }
}

#[derive(Clone, Copy)]
pub enum TextureFiltering
{
    MinMagPointMipmapPoint,
    MinMagLinearMipmapLinear,
    MinMagLinearMipmapPoint,
    MinMagPointMipmapLinear,
    MinPointMagLinearMipmapPoint,
    MinLinearMagPointMipmapLinear,
    Anisotropic
}

#[derive(Clone, Copy)]
pub enum TextureAddressing
{
    ClampToEdge,
    MirroredRepeat,
    Repeat
}

pub struct SamplerStatement
{
    pub name: String,
    pub filter_func: TextureFiltering,
    pub address_mode_u: TextureAddressing,
    pub address_mode_v: TextureAddressing,
    pub address_mode_w: TextureAddressing,
    pub anisotropic_level: u32,
    pub min_lod: f32,
    pub max_lod: f32
}

impl VarlistStatement for SamplerStatement
{
    fn new(name: String) -> Self
    {
        return SamplerStatement
        {
            name: name,
            filter_func: TextureFiltering::MinMagPointMipmapPoint,
            address_mode_u: TextureAddressing::ClampToEdge,
            address_mode_v: TextureAddressing::ClampToEdge,
            address_mode_w: TextureAddressing::ClampToEdge,
            anisotropic_level: 0,
            min_lod: f32::MIN,
            max_lod: f32::MAX
        }
    }
}

pub enum Statement
{
    Constant(Property),
    ConstantBuffer(Struct),
    Output(Property),
    VertexFormat(Struct),
    Pipeline(PipelineStatement),
    Blendfunc(BlendfuncStatement),
    Sampler(SamplerStatement),
    Noop // Used to represent a statement to ignore in the parse tree
}

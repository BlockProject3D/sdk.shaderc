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

use std::vec::Vec;

use bpx::sd::{Array, Object};

use crate::sal::ast::{BaseType, BlendfuncStatement, PipelineStatement, Property, PropertyType, Statement, Struct};

/* Op name | Op code | Nb arguments | Desc
 * NOOP    | 0x1     | 0            | No operation
 * CRT     | 0x2     | 2            | Create a new object using parameters pushed from the stack
 * PUSHU   | 0x3     | 1            | Push a new u8 parameter on the stack
 */

pub enum ObjectType
{
    Constant,
    ConstantBuffer,
    Output,
    VertexFormat,
    Pipeline,
    BlendFunc
}

fn encode_base_type(bt: BaseType) -> u8
{
    return match bt {
        BaseType::Int => 0x1,
        BaseType::Float => 0x2,
        BaseType::Uint => 0x3,
        BaseType::Bool => 0x4,
        BaseType::Double => 0x5
    };
}

fn encode_prop_type(pt: PropertyType) -> u32
{
    return match pt {
        PropertyType::Scalar(bt) => 0x0 >> 2 + encode_base_type(bt) >> 1,
        PropertyType::Vector(bt, l) => 0x1 >> 2 + encode_base_type(bt) >> 1 + l,
        PropertyType::Matrix(bt, l) => 0x2 >> 2 + encode_base_type(bt) >> 1 + l,
        PropertyType::Sampler => 0x3 >> 2,
        PropertyType::Texture2D(bt, l) => 0x4 >> 2 + encode_base_type(bt) >> 1 + l,
        PropertyType::Texture3D(bt, l) => 0x5 >> 2 + encode_base_type(bt) >> 1 + l,
        PropertyType::Texture2DArray(bt, l) => 0x6 >> 2 + encode_base_type(bt) >> 1 + l,
        PropertyType::TextureCube(bt, l) => 0x7 >> 2 + encode_base_type(bt) >> 1 + l
    };
}

fn encode_property(p: Property) -> Object
{
    let mut prop = Object::new();
    prop.set("Name", p.pname.into());
    prop.set("Type", encode_prop_type(p.ptype).into());
    return prop;
}

fn encode_struct(st: Struct) -> Object
{
    let mut o = Object::new();
    let mut a = Array::new();
    o.set("Name", st.name.into());
    for v in st.properties {
        a.add(encode_property(v).into());
    }
    o.set("Properties", a.into());
    return o;
}

fn encode_pipeline(pipeline: PipelineStatement) -> Object
{
    let mut o = Object::new();
    o.set("Name", pipeline.name.into());
    o.set("DepthEnable", pipeline.depth_enable.into());
    o.set("DepthWriteEnable", pipeline.depth_write_enable.into());
    o.set("ScissorEnable", pipeline.scissor_enable.into());
    o.set("RenderMode", (pipeline.render_mode as u8).into());
    o.set("CullingMode", (pipeline.culling_mode as u8).into());
    let mut a = Array::new();
    for (out, blend) in pipeline.blend_functions {
        let mut o = Object::new();
        o.set("Output", out.into());
        o.set("Blend", blend.into());
        a.add(o.into());
    }
    o.set("BlendFunctions", a.into());
    return o;
}

fn encode_blendfunc(blendfunc: BlendfuncStatement) -> Object
{
    let mut o = Object::new();
    o.set("Name", blendfunc.name.into());
    o.set("SrcColor", (blendfunc.src_color as u8).into());
    o.set("DstColor", (blendfunc.dst_color as u8).into());
    o.set("SrcAlpha", (blendfunc.src_alpha as u8).into());
    o.set("DstAlpha", (blendfunc.dst_alpha as u8).into());
    o.set("ColorOp", (blendfunc.color_op as u8).into());
    o.set("AlphaOp", (blendfunc.alpha_op as u8).into());
    return o;
}

pub fn compile(root: Vec<Statement>) -> Vec<(ObjectType, Object)>
{
    let mut lst = Vec::new();
    for v in root {
        match v {
            Statement::Constant(p) => lst.push((ObjectType::Constant, encode_property(p))),
            Statement::ConstantBuffer(st) => lst.push((ObjectType::ConstantBuffer, encode_struct(st))),
            Statement::Output(p) => lst.push((ObjectType::Output, encode_property(p))),
            Statement::VertexFormat(st) => lst.push((ObjectType::VertexFormat, encode_struct(st))),
            Statement::Pipeline(p) => lst.push((ObjectType::Pipeline, encode_pipeline(p))),
            Statement::Blendfunc(b) => lst.push((ObjectType::BlendFunc, encode_blendfunc(b))),
            Statement::Noop => ()
        }
    }
    return lst;
}

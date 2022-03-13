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
use bpx::shader::ShaderPack;

pub struct SymbolWriter<T: std::io::Write + std::io::Seek>
{
    inner: ShaderPack<T>,
    map: HashMap<String, u16>
}

impl<T: std::io::Write + std::io::Seek> SymbolWriter<T> {
    pub fn new(inner: ShaderPack<T>) -> SymbolWriter<T>
    {
        SymbolWriter {
            inner,
            map: HashMap::new()
        }
    }

    pub fn write(&mut self, builder: bpx::shader::symbol::Builder) -> bpx::shader::Result<()> {
        let s = builder.build();
        let name = s.name.clone();
        let mut symbols = self.inner.symbols_mut()
            .ok_or(bpx::shader::error::Error::Open(bpx::core::error::OpenError::SectionNotLoaded))?;
        let index = symbols.create(s)?;
        self.map.insert(name, index as _);
        Ok(())
    }

    pub fn lookup<T1: AsRef<str>>(&self, name: T1) -> u16
    {
        self.map[name.as_ref()]
    }

    pub fn into_inner(self) -> ShaderPack<T> {
        self.inner
    }
}

macro_rules! append_stages {
    ($var: ident > $builder: ident) => {
        if $var.stage_pixel {
            $builder.stage(Stage::Pixel);
        }
        if $var.stage_domain {
            $builder.stage(Stage::Domain);
        }
        if $var.stage_hull {
            $builder.stage(Stage::Hull);
        }
        if $var.stage_vertex {
            $builder.stage(Stage::Vertex);
        }
        if $var.stage_geometry {
            $builder.stage(Stage::Geometry);
        }
    };
}

pub(crate) use append_stages;
use bp3d_sal::ast::tree::{PipelineStatement, PropertyType, Struct};
use bp3d_symbols::{ArrayItemType, ConstantObject, OutputObject, PipelineObject, PropObject, PropType, StructObject, TextureObject, TextureObjectType};
use crate::targets::layout140::{size_of_base_type, StructOffset};

pub trait ToObject<T = ()> where Self: Sized
{
    type Object: bp3d_symbols::ToBpx;
    type Context;

    fn to_object(self, ctx: &Self::Context) -> Option<Self::Object>;

    fn to_bpx_object(self, debug: bool, ctx: &Self::Context) -> Result<bpx::sd::Value, bpx::sd::serde::Error> {
        use bp3d_symbols::ToBpx;
        match self.to_object(ctx) {
            None => Ok(bpx::sd::Value::Null),
            Some(v) => v.to_bpx(debug)
        }
    }
}

impl ToObject for ConstantObject {
    type Object = Self;
    type Context = ();

    fn to_object(self, _: &Self::Context) -> Option<Self::Object> {
        Some(self)
    }
}

impl ToObject for PropertyType<usize>
{
    type Object = TextureObject;
    type Context = ();

    fn to_object(self, _: &()) -> Option<Self::Object> {
        match self {
            PropertyType::Texture2D(value) => Some(TextureObject {
                ty: TextureObjectType::T2D,
                value
            }),
            PropertyType::Texture3D(value) => Some(TextureObject {
                ty: TextureObjectType::T3D,
                value
            }),
            PropertyType::Texture2DArray(value) => Some(TextureObject {
                ty: TextureObjectType::T2DArray,
                value
            }),
            PropertyType::TextureCube(value) => Some(TextureObject {
                ty: TextureObjectType::TCube,
                value
            }),
            _ => None
        }
    }
}

impl ToObject for OutputObject {
    type Object = Self;
    type Context = ();

    fn to_object(self, _: &Self::Context) -> Option<Self::Object> {
        Some(self)
    }
}

impl ToObject for PipelineStatement
{
    type Object = PipelineObject;
    type Context = ();

    fn to_object(self, _: &Self::Context) -> Option<Self::Object> {
        Some(PipelineObject {
            depth_enable: self.depth_enable,
            depth_write_enable: self.depth_write_enable,
            scissor_enable: self.scissor_enable,
            render_mode: self.render_mode,
            culling_mode: self.culling_mode
        })
    }
}

pub trait PropTypeExt
{
    fn get_size(&self) -> u32;
}

impl PropTypeExt for PropType
{
    fn get_size(&self) -> u32 //Returns an estimated size of the property for use
    // with vertex formats.
    {
        match self {
            PropType::Scalar(v) => size_of_base_type(*v) as u32,
            PropType::Vector(v) => size_of_base_type(v.item) as u32 * v.size as u32,
            PropType::Matrix(v) => size_of_base_type(v.item) as u32 * v.size as u32 * v.size as u32,
            _ => 0
        }
    }
}

fn new_prop_type<T: std::io::Seek + std::io::Write>(prop: PropertyType<usize>, syms: &SymbolWriter<T>, packed_structs: &Vec<StructOffset>) -> PropType
{
    match prop {
        PropertyType::Scalar(v) => PropType::Scalar(v),
        PropertyType::Vector(v) => PropType::Vector(v),
        PropertyType::Matrix(v) => PropType::Matrix(v),
        PropertyType::StructRef(v) => PropType::StructRef(syms.lookup(&packed_structs[v].name)),
        PropertyType::Array(v) => PropType::Array {
            size: v.size,
            ty: match v.item {
                bp3d_sal::ast::tree::ArrayItemType::Vector(v) => ArrayItemType::Vector(v),
                bp3d_sal::ast::tree::ArrayItemType::Matrix(v) => ArrayItemType::Matrix(v),
                bp3d_sal::ast::tree::ArrayItemType::StructRef(v) => ArrayItemType::StructRef(syms.lookup(&packed_structs[v].name))
            }
        },
        _ => unsafe { std::hint::unreachable_unchecked() } //That one should never trigger
        // if it does then there is a huge problem in the SAL processor
        // which forbids constant buffers with samplers and similar types
    }
}

fn new_prop_type_simple(prop: PropertyType<usize>) -> PropType
{
    match prop {
        PropertyType::Scalar(v) => PropType::Scalar(v),
        PropertyType::Vector(v) => PropType::Vector(v),
        PropertyType::Matrix(v) => PropType::Matrix(v),
        _ => panic!("Attempted to allocate a broken PropType")
    }
}


impl<'a, T: 'a + std::io::Write + std::io::Seek> ToObject<T> for &'a StructOffset
{
    type Object = StructObject;
    type Context = (&'a SymbolWriter<T>, &'a Vec<StructOffset>);

    fn to_object(self, (syms, packed_structs): &(&'a SymbolWriter<T>, &'a Vec<StructOffset>)) -> Option<Self::Object> {
        Some(StructObject {
            size: self.size as _,
            props: self.props.iter().map(|v| PropObject {
                name: v.inner.pname.clone(),
                offset: v.aligned_offset as _,
                ty: new_prop_type(v.inner.ptype, syms, packed_structs)
            }).collect()
        })
    }
}

impl ToObject for Struct<usize>
{
    type Object = StructObject;
    type Context = ();

    fn to_object(self, _: &()) -> Option<Self::Object> {
        let mut st = StructObject {
            size: 0,
            props: Vec::new()
        };
        for prop in self.props {
            let ty = new_prop_type_simple(prop.ptype);
            let size = ty.get_size();
            st.props.push(PropObject {
                name: prop.pname,
                ty,
                offset: st.size
            });
            st.size += size;
        }
        Some(st)
    }
}

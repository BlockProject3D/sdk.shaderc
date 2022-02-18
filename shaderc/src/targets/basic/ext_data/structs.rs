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

use serde::Deserialize;
use serde::Serialize;
use sal::ast::tree::{BaseType, PropertyType, Struct, VectorType};
use crate::targets::basic::ext_data::SymbolWriter;
use crate::targets::layout140::{size_of_base_type, StructOffset};
use super::ToObject;

#[derive(Serialize, Deserialize)]
pub enum ArrayItemType
{
    Vector(VectorType),
    Matrix(VectorType),
    StructRef(u16), //Index of referenced symbol in symbol table.
}

#[derive(Serialize, Deserialize)]
pub enum PropType
{
    Scalar(BaseType),
    Vector(VectorType),
    Matrix(VectorType),
    StructRef(u16), //Index of referenced symbol in symbol table.
    Array {
        size: u32,
        ty: ArrayItemType
    }
}

impl PropType
{
    pub fn get_size(&self) -> u32 //Returns an estimated size of the property for use
    // with vertex formats.
    {
        match self {
            PropType::Scalar(v) => size_of_base_type(*v) as u32,
            PropType::Vector(v) => size_of_base_type(v.item) as u32 * v.size as u32,
            PropType::Matrix(v) => size_of_base_type(v.item) as u32 * v.size as u32 * v.size as u32,
            _ => 0
        }
    }

    fn new<T: std::io::Seek + std::io::Write>(prop: PropertyType, syms: &SymbolWriter<T>) -> PropType
    {
        match prop {
            PropertyType::Scalar(v) => PropType::Scalar(v),
            PropertyType::Vector(v) => PropType::Vector(v),
            PropertyType::Matrix(v) => PropType::Matrix(v),
            PropertyType::StructRef(v) => PropType::StructRef(syms.lookup(v)),
            PropertyType::Array(v) => PropType::Array {
                size: v.size,
                ty: match v.item {
                    sal::ast::tree::ArrayItemType::Vector(v) => ArrayItemType::Vector(v),
                    sal::ast::tree::ArrayItemType::Matrix(v) => ArrayItemType::Matrix(v),
                    sal::ast::tree::ArrayItemType::StructRef(v) => ArrayItemType::StructRef(syms.lookup(v))
                }
            },
            _ => unsafe { std::hint::unreachable_unchecked() } //That one should never trigger
            // if it does then there is a huge problem in the SAL processor
            // which forbids constant buffers with samplers and similar types
        }
    }

    fn new_simple(prop: PropertyType) -> PropType
    {
        match prop {
            PropertyType::Scalar(v) => PropType::Scalar(v),
            PropertyType::Vector(v) => PropType::Vector(v),
            PropertyType::Matrix(v) => PropType::Matrix(v),
            _ => panic!("Attempted to allocate a broken PropType")
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PropObject
{
    pub name: String,
    pub offset: u32,
    pub ty: PropType
}

#[derive(Serialize, Deserialize)]
pub struct StructObject
{
    pub size: u32,
    pub props: Vec<PropObject>
}

impl<T: std::io::Write + std::io::Seek> ToObject<T> for StructOffset
{
    type Object = StructObject;
    type Context = SymbolWriter<T>;

    fn to_object(self, ctx: &SymbolWriter<T>) -> Option<Self::Object> {
        Some(StructObject {
            size: self.size as _,
            props: self.props.into_iter().map(|v| PropObject {
                name: v.inner.pname,
                offset: v.aligned_offset as _,
                ty: PropType::new(v.inner.ptype, ctx)
            }).collect()
        })
    }
}

impl ToObject for Struct
{
    type Object = StructObject;
    type Context = ();

    fn to_object(self, _: &()) -> Option<Self::Object> {
        let mut st = StructObject {
            size: 0,
            props: Vec::new()
        };
        for prop in self.props {
            let ty = PropType::new_simple(prop.ptype);
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

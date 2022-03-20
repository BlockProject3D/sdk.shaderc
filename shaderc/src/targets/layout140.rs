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

use std::ops::{Deref, DerefMut};
use log::{error, warn};
use bp3d_sal::ast::tree::{ArrayItemType, Attribute, BaseType, Property, PropertyType, Struct};
use thiserror::Error;

// STD140 layout rules for paddings
// https://www.khronos.org/registry/OpenGL/specs/gl/glspec46.core.pdf
// Section 7.6.2.2

#[derive(Debug, Error)]
pub enum Error {
    #[error("attempt to reference undeclared packed struct")]
    Undeclared
}

pub fn size_of_base_type(t: BaseType) -> usize
{
    match t {
        BaseType::Int => 4,
        BaseType::Float => 4,
        BaseType::Uint => 4,
        BaseType::Bool => 4,
        BaseType::Double => 8,
    }
}

fn base_alignment(p: &PropertyType<usize>) -> usize
{
    match p {
        PropertyType::Scalar(t) => size_of_base_type(*t),
        PropertyType::Vector(v) => {
            match v.size {
                2 => 2 * size_of_base_type(v.item),
                3 | 4 => 4 * size_of_base_type(v.item),
                _ => 0
            }
        },
        PropertyType::Matrix(m) => 4 * size_of_base_type(m.item),
        _ => 0
    }
}

fn array_base_alignment(a: &ArrayItemType<usize>) -> usize
{
    match a {
        ArrayItemType::Vector(v) => 4 * size_of_base_type(v.item),
        ArrayItemType::Matrix(m) => 4 * size_of_base_type(m.item),
        _ => 0
    }
}

pub fn size_of(p: &PropertyType<usize>) -> usize
{
    match p {
        PropertyType::Scalar(b) => size_of_base_type(*b),
        PropertyType::Vector(v) => size_of_base_type(v.item) * v.size as usize,
        PropertyType::Matrix(m) => size_of_base_type(m.item) * m.size as usize * m.size as usize,
        _ => {
            warn!("Attempted to compute size of handle object; object handles are not permitted in constant buffers!");
            0
        }
    }
}

pub fn array_size_of(p: &ArrayItemType<usize>) -> usize
{
    match p {
        ArrayItemType::Vector(v) => size_of_base_type(v.item) * v.size as usize,
        ArrayItemType::Matrix(m) => size_of_base_type(m.item) * m.size as usize * m.size as usize,
        _ => {
            warn!("Attempted to compute size of handle object; object handles are not permitted in constant buffers!");
            0
        }
    }
}

pub struct Offset<T>
{
    pub inner: T,
    pub aligned_offset: usize,
    pub offset: usize,
    pub size: usize,
    pub base_alignment: usize
}

impl<T> Deref for Offset<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        &self.inner
    }
}

impl<T> DerefMut for Offset<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.inner
    }
}

fn round_to_vec4(base_alignment: usize) -> usize
{
    let vec4 = if base_alignment > 16 { 32 } else { 16 };
    round_to_base_alignment(base_alignment, vec4)
}

fn round_to_base_alignment(mut size: usize, base_alignment: usize) -> usize
{
    while size % base_alignment != 0 {
        size += 1;
    }
    size
}

pub struct StructOffset
{
    pub name: String,
    pub attr: Option<Attribute>,
    pub props: Vec<Offset<Property<usize>>>,
    pub size: usize,
    pub base_alignment: usize
}

pub fn compile_struct(st: Struct<usize>, packed_structs: &Vec<StructOffset>) -> Result<StructOffset, Error>
{
    let mut props = Vec::new();
    let mut cur_size = 0;
    let mut cur_offset: usize = 0;
    let mut max_base_alignment = 0;
    for v in st.props {
        let (base_alignment, size) = match &v.ptype {
            PropertyType::StructRef(s) => {
                let st = packed_structs.get(*s).ok_or_else(|| {
                    error!("Couldn't find referenced struct '{}', is it declared in the right order?", s);
                    Error::Undeclared
                })?;
                (round_to_vec4(st.base_alignment), st.size)
            },
            PropertyType::Array(a) => {
                match &a.item {
                    ArrayItemType::StructRef(s) => {
                        let st = packed_structs.get(*s).ok_or_else(|| {
                            error!("Couldn't find referenced struct '{}', is it declared in the right order?", s);
                            Error::Undeclared
                        })?;
                        (round_to_vec4(st.base_alignment), a.size as usize * st.size)
                    },
                    _ => (array_base_alignment(&a.item), a.size as usize * array_size_of(&a.item))
                }
            }
            _ => (base_alignment(&v.ptype), size_of(&v.ptype))
        };
        if max_base_alignment == 0 || base_alignment > max_base_alignment {
            max_base_alignment = base_alignment;
        }
        let offset = cur_offset;
        let aligned_offset = round_to_base_alignment(offset, base_alignment);
        let offsetprop = Offset {
            inner: v,
            offset,
            aligned_offset,
            base_alignment,
            size
        };
        cur_offset += size;
        cur_size += size;
        props.push(offsetprop);
    }
    Ok(StructOffset {
        size: round_to_base_alignment(cur_size, max_base_alignment),
        base_alignment: max_base_alignment,
        attr: st.attr,
        name: st.name,
        props
    })
}

pub fn compile_packed_structs(mut packed_structs: Vec<Struct<usize>>) -> Result<Vec<StructOffset>, Error>
{
    let mut vec = Vec::new();
    while packed_structs.len() > 0 {
        let st = packed_structs.remove(0);
        let compiled = compile_struct(st, &vec)?;
        vec.push(compiled);
    }
    Ok(vec)
}

#[cfg(test)]
mod tests
{
    use bp3d_sal::ast::tree::{ArrayItemType, ArrayType, Attribute, BaseType, Property, PropertyType, Struct, VectorType};
    use crate::targets::layout140::{compile_packed_structs, compile_struct};

    #[test]
    fn basic()
    {
        let lighting = Struct {
            name: "Lighting".into(),
            attr: Some(Attribute::Order(2)),
            props: vec![
                Property {
                    pname: "Count".into(),
                    ptype: PropertyType::Scalar(BaseType::Uint),
                    pattr: None
                },
                Property {
                    pname: "Lights".into(),
                    ptype: PropertyType::Array(ArrayType {
                        size: 32,
                        item: ArrayItemType::StructRef(0)
                    }),
                    pattr: None
                }
            ]
        };
        let light = Struct {
            name: "Light".into(),
            attr: Some(Attribute::Pack),
            props: vec![
                Property {
                    pname: "Color".into(),
                    ptype: PropertyType::Vector(VectorType {
                        size: 4,
                        item: BaseType::Float
                    }),
                    pattr: None
                },
                Property {
                    pname: "Attenuation".into(),
                    ptype: PropertyType::Scalar(BaseType::Float),
                    pattr: None
                }
            ]
        };
        let packed_structs = vec!(light);
        let packed_compiled = compile_packed_structs(packed_structs).unwrap();
        let compiled = compile_struct(lighting, &packed_compiled).unwrap();
        assert_eq!(compiled.size, 1040); //The size of the compiled structure includes ALL required alignments
        assert_eq!(compiled.base_alignment, 16);
        let aligned_offsets: Vec<usize> = compiled.props.iter().map(|v| v.aligned_offset).collect();
        assert_eq!(aligned_offsets, vec![0, 16]);
        let aligned_offsets: Vec<usize> = packed_compiled[0].props.iter().map(|v| v.aligned_offset).collect();
        assert_eq!(aligned_offsets, vec![0, 16]);
    }
}

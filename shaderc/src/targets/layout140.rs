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

use log::{debug, error, warn};
use sal::ast::tree::{Attribute, BaseType, Property, PropertyType, Struct, VectorType};
use crate::options::Error;

// STD140 layout rules for paddings
// https://www.khronos.org/registry/OpenGL/specs/gl/glspec46.core.pdf
// Section 7.6.2.2

fn size_of_base_type(t: BaseType) -> usize
{
    match t {
        BaseType::Int => 4,
        BaseType::Float => 4,
        BaseType::Uint => 4,
        BaseType::Bool => 4,
        BaseType::Double => 8,
    }
}

fn base_alignment(p: &PropertyType) -> usize
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

pub fn size_of(p: &PropertyType) -> usize
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

pub fn offset_of(c: &Property, layout: &Struct) -> usize
{
    let mut flag = false;
    let mut offset: usize = 0;
    for v in &layout.props {
        let size = size_of(&v.ptype);
        if size == 0 {
            warn!("Property '{}' in layout '{}' is zero-sized!", c.pname, layout.name);
        }
        if v.pname == c.pname {
            flag = true;
            break;
        }
        offset += size;
    }
    if !flag {
        warn!("Unable to locate property '{}' in layout '{}'", c.pname, layout.name);
    }
    offset
}

pub fn aligned_offset_of(c: &Property, layout: &Struct) -> usize
{
    let mut flag = false;
    let mut offset: usize = 0;
    for v in &layout.props {
        let size = size_of(&v.ptype);
        let align = base_alignment(&v.ptype);
        if size == 0 {
            warn!("Property '{}' in layout '{}' is zero-sized!", c.pname, layout.name);
            continue;
        }
        if v.pname == c.pname {
            flag = true;
            break;
        }
        offset += size;
        while offset % align != 0 {
            offset += 1;
        }
    }
    if !flag {
        warn!("Unable to locate property '{}' in layout '{}'", c.pname, layout.name);
    }
    offset
}

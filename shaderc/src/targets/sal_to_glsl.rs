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

//TODO: Write the function to compile constant buffers and optimize layout for std140

use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use log::{debug, error, warn};
use sal::ast::tree::{Attribute, Property, PropertyType, Struct, VectorType};
use crate::options::Error;
use crate::targets::basic::{Slot, StmtDecomposition};

fn get_char(v: VectorType) -> char
{
    let c = v.item.get_char();
    if c == 'f' {
        ' '
    } else {
        c
    }
}

fn translate_property(p: &Property) -> String
{
    let ptype: Cow<str> = match p.ptype {
        PropertyType::Scalar(s) => s.get_name().into(),
        PropertyType::Vector(v) => format!("{}vec{}", get_char(v), v.size).into(),
        PropertyType::Matrix(m) => format!("{}mat{}", get_char(m), m.size).into(),
        PropertyType::Sampler => "".into(),
        PropertyType::Texture2D(_) => "sampler2D".into(),
        PropertyType::Texture3D(_) => "sampler3D".into(),
        PropertyType::Texture2DArray(_) => "sampler2DArray".into(),
        PropertyType::TextureCube(_) => "samplerCube".into()
    };
    if &ptype == "" {
        return String::default()
    }
    format!("{} {};", ptype, p.pname)
}

fn translate_cbuffer(explicit_bindings: bool, s: &Slot<Struct>) -> String
{
    let mut str;
    if explicit_bindings {
        str = format!("layout (binding = {}, std140) uniform {} {{", s.slot, s.inner.name);
    } else {
        str = format!("layout (std140) uniform {} {{", s.inner.name);
    }
    for v in &s.inner.props {
        let prop = Property {
            pattr: None,
            pname: [&*s.inner.name, &*v.pname].join("_"),
            ptype: v.ptype
        };
        str.push_str(&translate_property(&prop));
    }
    str.push_str("};");
    str
}

fn translate_vformat(s: &Struct) -> String
{
    let mut str= String::new();
    for (loc, v) in s.props.iter().enumerate() {
        let prop = Property {
            pattr: None,
            pname: [&*s.name, &*v.pname].join("_"),
            ptype: v.ptype
        };
        str.push_str(&format!("layout (location = {}) in {}", loc, translate_property(&prop)));
    }
    str
}

fn translate_outputs(outputs: &Vec<Slot<Property>>) -> Result<String, Error>
{
    let mut str= String::new();
    let mut set = HashSet::new();
    for v in outputs.iter() {
        if !set.insert(v.slot) {
            return Err(Error::from(format!("multiple definition of output slot {}", v.slot)))
        }
        str.push_str(&format!("layout (location = {}) out {}", v.slot, translate_property(&v.inner)));
    }
    Ok(str)
}

fn size_of(p: &Property) -> usize
{
    match p.ptype {
        PropertyType::Scalar(_) => 16, //layout (std140) requires all fields to be aligned to 16 bytes boundaries
        PropertyType::Vector(_) => 16,
        PropertyType::Matrix(m) => m.size as usize * 16,
        PropertyType::Sampler => 0,
        PropertyType::Texture2D(_) => 0,
        PropertyType::Texture3D(_) => 0,
        PropertyType::Texture2DArray(_) => 0,
        PropertyType::TextureCube(_) => 0
    }
}

fn offset_of(c: &Property, layout: &Struct) -> usize
{
    let mut flag = false;
    let mut offset: usize = 0;
    for v in &layout.props {
        let size = size_of(v);
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

fn translate_root_consts(explicit_bindings: bool, root_constants_layout: &Struct, consts: &Vec<Slot<Property>>) -> String
{
    if consts.is_empty() {
        return String::default();
    }
    let mut str;
    if explicit_bindings {
        str = String::from("layout (binding = 0, std140) uniform __Root {");
    } else {
        str = String::from("layout (std140) uniform __Root {");
    }
    let mut last_offset: usize = 0;
    for (i, v) in root_constants_layout.props.iter().enumerate() {
        if consts.iter().any(|p| p.inner.pname == v.pname) {
            let offset = offset_of(v, root_constants_layout);
            let size = size_of(v);
            let padding_size = (offset - last_offset) / 16; //obtain number of vec4f to pad
            debug!("Property '{}': offset = {}, size = {}", v.pname, offset, size);
            if padding_size > 0 {
                str.push_str(&format!("vec4 __padding{}__[{}];", i, padding_size));
            }
            last_offset = offset + size;
            str.push_str(&translate_property(v));
        }
    }
    str.push_str("};");
    str
}

fn test_cbuffers_unique_slots(cbuffers: &Vec<Slot<Struct>>) -> Result<(), Error>
{
    let mut set = HashSet::new();
    // Extract duplicate binding slots
    let flag = cbuffers.iter().any(|s| {
        if set.contains(&s.slot) {
            error!("Duplicate slot binding {}", s.slot);
            return true;
        } else {
            set.insert(s.slot);
        }
        false
    });
    if flag { //Oh now we've got duplicate binding slots => terminate compilation immediately
        return Err(Error::new("duplicate slot bindings in one or more constant buffer declaration"));
    }
    Ok(())
}

pub fn translate_sal_to_glsl(explicit_bindings: bool, root_constants_layout: &Struct, sal: &StmtDecomposition) -> Result<String, Error>
{
    let vformat = sal.vformat.as_ref().map(|s| translate_vformat(&s)).unwrap_or_default();
    let constants = translate_root_consts(explicit_bindings, root_constants_layout, &sal.root_constants);
    let outputs = translate_outputs(&sal.outputs)?;
    test_cbuffers_unique_slots(&sal.cbuffers)?;
    let cbuffers: Vec<String> = sal.cbuffers.iter().map(|s| translate_cbuffer(explicit_bindings, s)).collect();
    let cbuffers = cbuffers.join("\n");
    let objects: Vec<String> = sal.objects.iter().filter_map(|p| {
        let sji = translate_property(&p.inner);
        if !sji.is_empty() {
            if explicit_bindings {
                Some(format!("layout (binding = {}) uniform {}", p.slot, sji))
            } else {
                Some(format!("uniform {}", sji))
            }
        } else {
            None
        }
    }).collect();
    let objects = objects.join("\n");
    debug!("translated vertex format: {}", vformat);
    debug!("translated root constants: {}", constants);
    debug!("translated outputs: {}", outputs);
    debug!("translated constant buffers: {}", cbuffers);
    debug!("translated objects: {}", objects);
    let output = [&*vformat, &*constants, &*outputs, &*cbuffers, &*objects].iter()
        .map(|s| *s)
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");
    Ok(output)
}

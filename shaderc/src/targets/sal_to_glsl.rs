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

use std::borrow::Cow;
use std::collections::HashSet;
use log::{debug, error};
use bp3d_sal::ast::tree::{ArrayItemType, Property, PropertyType, Struct, VectorType};
use crate::options::Error;
use crate::targets::basic::{BasicAst, Slot, StmtDecomposition};

fn get_char(v: VectorType) -> char
{
    let c = v.item.get_char();
    if c == 'f' {
        ' '
    } else {
        c
    }
}

fn translate_property(p: &Property<usize>, ast: &BasicAst) -> String
{
    let mut array = None;
    let ptype: Cow<str> = match &p.ptype {
        PropertyType::Scalar(s) => s.get_name().into(),
        PropertyType::Vector(v) => format!("{}vec{}", get_char(*v), v.size).into(),
        PropertyType::Matrix(m) => format!("{}mat{}", get_char(*m), m.size).into(),
        PropertyType::Sampler => "".into(),
        PropertyType::Texture2D(_) => "sampler2D".into(),
        PropertyType::Texture3D(_) => "sampler3D".into(),
        PropertyType::Texture2DArray(_) => "sampler2DArray".into(),
        PropertyType::TextureCube(_) => "samplerCube".into(),
        PropertyType::StructRef(s) => (&* ast.get_struct_ref(*s).name).into(),
        PropertyType::Array(a) => {
            let item: Cow<str> = match &a.item {
                ArrayItemType::Vector(v) => format!("{}vec{}", get_char(*v), v.size).into(),
                ArrayItemType::Matrix(m) => format!("{}mat{}", get_char(*m), m.size).into(),
                ArrayItemType::StructRef(s) => (&* ast.get_struct_ref(*s).name).into()
            };
            array = Some(a.size);
            format!("{}", item).into()
        }
    };
    if &ptype == "" {
        return String::default()
    }
    if let Some(size) = array {
        format!("{} {}[{}];", ptype, p.pname, size)
    } else {
        format!("{} {};", ptype, p.pname)
    }
}

fn translate_packed_struct(s: &Struct<usize>, ast: &BasicAst) -> String
{
    let mut str= format!("struct {} {{", s.name);
    for v in &s.props {
        str.push_str(&translate_property(v, ast));
    }
    str.push_str("};");
    str
}

fn translate_cbuffer(explicit_bindings: bool, s: &Slot<Struct<usize>>, ast: &BasicAst) -> String
{
    let mut str;
    if explicit_bindings {
        str = format!("layout (binding = {}, std140) uniform {} {{", s.slot.get(), s.inner.name);
    } else {
        str = format!("layout (std140) uniform {} {{", s.inner.name);
    }
    for v in &s.inner.props {
        let prop = Property {
            pattr: None,
            pname: [&*s.inner.name, &*v.pname].join("_"),
            ptype: v.ptype.clone()
        };
        str.push_str(&translate_property(&prop, ast));
    }
    str.push_str("};");
    str
}

fn translate_vformat(s: &Struct<usize>, ast: &BasicAst) -> String
{
    let mut str= String::new();
    for (loc, v) in s.props.iter().enumerate() {
        let prop = Property {
            pattr: None,
            pname: [&*s.name, &*v.pname].join("_"),
            ptype: v.ptype.clone()
        };
        str.push_str(&format!("layout (location = {}) in {}", loc, translate_property(&prop, ast)));
    }
    str
}

fn translate_outputs(ast: &BasicAst) -> Result<String, Error>
{
    let mut str= String::new();
    let mut set = HashSet::new();
    for v in ast.outputs.iter() {
        if !set.insert(v.slot.get()) {
            return Err(Error::from(format!("multiple definition of output slot {}", v.slot.get())))
        }
        str.push_str(&format!("layout (location = {}) out {}", v.slot.get(), translate_property(&v.inner, ast)));
    }
    Ok(str)
}

fn translate_root_consts(explicit_bindings: bool, root_constants_layout: &Struct<usize>, ast: &BasicAst) -> String
{
    if ast.root_constants.is_empty() {
        return String::default();
    }
    let mut str;
    if explicit_bindings {
        str = String::from("layout (binding = 0, std140) uniform __Root {");
    } else {
        str = String::from("layout (std140) uniform __Root {");
    }
    let last_used_prop = root_constants_layout.props.iter().rfind(|p| {
        if ast.root_constants.iter().any(|v| &v.inner == *p) {
            true
        } else {
            false
        }
    }).unwrap(); //SAFETY: unwrap cannot fail otherwise their exists no constants in the root constant buffer
    // but in this case the very first if block in this function would have triggered
    for v in root_constants_layout.props.iter() {
        str.push_str(&translate_property(v, ast));
        //No more root constants in the root constants layout are used in the shader: stop generation
        if v == last_used_prop {
            break;
        }
    }
    str.push_str("};");
    str
}

fn test_cbuffers_unique_slots(ast: &BasicAst) -> Result<(), Error>
{
    let mut set = HashSet::new();
    // Extract duplicate binding slots
    let flag = ast.cbuffers.iter().any(|s| {
        if set.contains(&s.slot.get()) {
            error!("Duplicate slot binding {}", s.slot.get());
            return true;
        } else {
            set.insert(s.slot.get());
        }
        false
    });
    if flag { //Oh now we've got duplicate binding slots => terminate compilation immediately
        return Err(Error::new("duplicate slot bindings in one or more constant buffer declaration"));
    }
    Ok(())
}

pub fn translate_sal_to_glsl(explicit_bindings: bool, root_constants_layout: &Struct<usize>, ast: &BasicAst) -> Result<String, Error>
{
    let vformat = ast.vformat.as_ref().map(|s| translate_vformat(&s, ast)).unwrap_or_default();
    let constants = translate_root_consts(explicit_bindings, root_constants_layout, ast);
    let outputs = translate_outputs(ast)?;
    test_cbuffers_unique_slots(ast)?;
    let structs: Vec<String> = ast.packed_structs.iter().map(|s| translate_packed_struct(s, ast)).collect();
    let structs = structs.join("\n");
    let cbuffers: Vec<String> = ast.cbuffers.iter().map(|s| translate_cbuffer(explicit_bindings, s, ast)).collect();
    let cbuffers = cbuffers.join("\n");
    let objects: Vec<String> = ast.objects.iter().filter_map(|p| {
        let sji = translate_property(&p.inner, ast);
        if !sji.is_empty() {
            if explicit_bindings {
                Some(format!("layout (binding = {}) uniform {}", p.slot.get(), sji))
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
    debug!("translated structures: {}", structs);
    debug!("translated constant buffers: {}", cbuffers);
    debug!("translated objects: {}", objects);
    let output = [&*vformat, &*constants, &*outputs, &*structs, &*cbuffers, &*objects].iter()
        .map(|s| *s)
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");
    Ok(output)
}

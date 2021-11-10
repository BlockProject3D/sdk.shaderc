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

use std::{path::PathBuf, string::String, vec::Vec};

use phf::phf_map;

use crate::ast::tree as ast;
use crate::parser::tree as tree;
use crate::utils::parse_file;

fn parse_vec_base(t: &str) -> Result<(ast::BaseType, u8), String>
{
    let len = match &t[3..t.len() - 1].parse::<u8>() {
        Err(e) => {
            return Err(format!(
                "[Shader Annotation Language] Invalid vector item count {}: {}",
                &t[3..t.len() - 1],
                e
            ))
        },
        Ok(v) => *v
    };
    let base_type = match &t[t.len() - 1..] {
        "f" => ast::BaseType::Float,
        "d" => ast::BaseType::Double,
        "u" => ast::BaseType::Uint,
        "i" => ast::BaseType::Int,
        "b" => ast::BaseType::Bool,
        _ => {
            return Err(format!(
                "[Shader Annotation Language] Unknown type letter {}",
                &t[t.len() - 1..]
            ))
        },
    };
    return Ok((base_type, len));
}

fn pasre_vec(t: &str) -> Result<(ast::BaseType, u8), String>
{
    if !t.starts_with("vec") {
        return Err(format!("[Shader Annotation Language] Unknown vector type {}", t));
    }
    let (base_type, len) = parse_vec_base(t)?;
    return Ok((base_type, len));
}

fn try_parse_matrix(t: &str) -> Result<Option<ast::PropertyType>, String>
{
    if !t.starts_with("mat") {
        return Ok(None);
    }
    let (base_type, len) = parse_vec_base(t)?;
    return Ok(Some(ast::PropertyType::Matrix(base_type, len)));
}

fn try_parse_texture(t: &str, subt: Option<&str>) -> Result<Option<ast::PropertyType>, String>
{
    if let Some(st) = subt {
        return match t {
            "Texture2D" => {
                let (t, c) = pasre_vec(st)?;
                Ok(Some(ast::PropertyType::Texture2D(t, c)))
            },
            "Texture3D" => {
                let (t, c) = pasre_vec(st)?;
                Ok(Some(ast::PropertyType::Texture3D(t, c)))
            },
            "Texture2DArray" => {
                let (t, c) = pasre_vec(st)?;
                Ok(Some(ast::PropertyType::Texture2DArray(t, c)))
            },
            "TextureCube" => {
                let (t, c) = pasre_vec(st)?;
                Ok(Some(ast::PropertyType::TextureCube(t, c)))
            },
            _ => Ok(None)
        };
    }
    return Ok(None);
}

fn parse_type(t: &str) -> Result<ast::PropertyType, String>
{
    let mut sub_type = None;
    if let Some(id) = t.find("::") {
        sub_type = Some(&t[id + 2..]);
    }
    return match t {
        "Sampler" => Ok(ast::PropertyType::Sampler),
        "float" => Ok(ast::PropertyType::Scalar(ast::BaseType::Float)),
        "double" => Ok(ast::PropertyType::Scalar(ast::BaseType::Double)),
        "int" => Ok(ast::PropertyType::Scalar(ast::BaseType::Int)),
        "uint" => Ok(ast::PropertyType::Scalar(ast::BaseType::Uint)),
        "bool" => Ok(ast::PropertyType::Scalar(ast::BaseType::Bool)),
        _ => {
            if let Some(elem) = try_parse_matrix(t)? {
                return Ok(elem);
            }
            if let Some(elem) = try_parse_texture(t, sub_type)? {
                return Ok(elem);
            }
            let (t, c) = pasre_vec(t)?;
            Ok(ast::PropertyType::Vector(t, c))
        }
    }
}

fn parse_prop(p: tree::Property) -> Result<ast::Property, String>
{
    let ptype = parse_type(&p.ptype)?;
    return Ok(ast::Property {
        ptype,
        pname: p.pname
    });
}

fn parse_struct(s: tree::Struct, err: &str) -> Result<ast::Struct, String>
{
    let mut plist = Vec::new();

    for v in s.props {
        let p = parse_prop(v)?;
        match p.ptype {
            ast::PropertyType::Sampler => {
                return Err(format!(
                    "[Shader Annotation Language] Sampler definitions are not allowed in a {}",
                    err
                ))
            },
            ast::PropertyType::Texture2D(_, _) => {
                return Err(format!(
                    "[Shader Annotation Language] Texture2D definitions are not allowed in a {}",
                    err
                ))
            },
            ast::PropertyType::Texture3D(_, _) => {
                return Err(format!(
                    "[Shader Annotation Language] Texture3D definitions are not allowed in a {}",
                    err
                ))
            },
            ast::PropertyType::Texture2DArray(_, _) => {
                return Err(format!(
                    "[Shader Annotation Language] Texture2DArray definitions are not allowed in a {}",
                    err
                ))
            },
            ast::PropertyType::TextureCube(_, _) => {
                return Err(format!(
                    "[Shader Annotation Language] TextureCube definitions are not allowed in a {}",
                    err
                ))
            },
            _ => ()
        };
        plist.push(p);
    }
    return Ok(ast::Struct {
        name: s.name,
        properties: plist
    });
}

static BLENDFACTOR: phf::Map<&'static str, ast::BlendFactor> = phf_map! {
    "ZERO" => ast::BlendFactor::Zero,
    "ONE" => ast::BlendFactor::One,
    "SRC_COLOR" => ast::BlendFactor::SrcColor,
    "ONE_MINUS_SRC_COLOR" => ast::BlendFactor::OneMinusSrcColor,
    "SRC_ALPHA" => ast::BlendFactor::SrcAlpha,
    "ONE_MINUS_SRC_ALPHA" => ast::BlendFactor::OneMinusSrcAlpha,
    "DST_COLOR" => ast::BlendFactor::DstColor,
    "ONE_MINUS_DST_COLOR" => ast::BlendFactor::OneMinusDstColor,
    "DST_ALPHA" => ast::BlendFactor::DstAlpha,
    "ONE_MINUS_DST_ALPHA" => ast::BlendFactor::OneMinusDstAlpha,
    "SRC_ALPHA_SATURATE" => ast::BlendFactor::SrcAlphaSaturate,
    "SRC1_COLOR" => ast::BlendFactor::Src1Color,
    "ONE_MINUS_SRC!_COLOR" => ast::BlendFactor::OneMinusSrc1Color,
    "SRC1_ALPHA" => ast::BlendFactor::Src1Alpha,
    "ONE_MINUS_SRC1_ALPHA" => ast::BlendFactor::OneMinusSrc1Alpha
};

static BLENDOP: phf::Map<&'static str, ast::BlendOperator> = phf_map! {
    "ADD" => ast::BlendOperator::Add,
    "SUBTRACT" => ast::BlendOperator::Subtract,
    "INV_SUBTRACT" => ast::BlendOperator::InverseSubtract,
    "MIN" => ast::BlendOperator::Min,
    "MAX" => ast::BlendOperator::Max
};

static RENDERMODE: phf::Map<&'static str, ast::RenderMode> = phf_map! {
    "TRIANGLES" => ast::RenderMode::Triangles,
    "WIREFRAME" => ast::RenderMode::Wireframe,
    "PATCHES" => ast::RenderMode::Patches
};

static CULLINGMODE: phf::Map<&'static str, ast::CullingMode> = phf_map! {
    "FRONT_FACE" => ast::CullingMode::FrontFace,
    "BACK_FACE" => ast::CullingMode::BackFace,
    "DISABLED" => ast::CullingMode::Disabled
};

fn parse_enum<T: Copy>(value: tree::Value, map: &phf::Map<&'static str, T>) -> Result<T, String>
{
    if let tree::Value::Identifier(id) = value {
        if let Some(e) = map.get(&*id) {
            return Ok(*e);
        }
        return Err(format!("[Shader Annotation Language] Unknown enum {}", id));
    }
    return Err(String::from("[Shader Annotation Language] Value is not an enumeration"));
}

type VarParseFunc<T> = fn(obj: &mut T, value: tree::Value) -> Result<(), String>;
type VarParseFallback<T> = fn(obj: &mut T, name: &str, value: tree::Value) -> Result<(), String>;

static VARLIST_BLENDFUNC: phf::Map<&'static str, VarParseFunc<ast::BlendfuncStatement>> = phf_map! {
    "SrcColor" => |obj, val|
    {
        let e = parse_enum(val, &BLENDFACTOR)?;
        obj.src_color = e;
        return Ok(());
    },
    "DstColor" => |obj, val|
    {
        let e = parse_enum(val, &BLENDFACTOR)?;
        obj.dst_color = e;
        return Ok(());
    },
    "SrcAlpha" => |obj, val|
    {
        let e = parse_enum(val, &BLENDFACTOR)?;
        obj.src_alpha = e;
        return Ok(());
    },
    "DstAlpha" => |obj, val|
    {
        let e = parse_enum(val, &BLENDFACTOR)?;
        obj.dst_alpha = e;
        return Ok(());
    },
    "ColorOp" => |obj, val|
    {
        let e = parse_enum(val, &BLENDOP)?;
        obj.color_op = e;
        return Ok(());
    },
    "AlphaOp" => |obj, val|
    {
        let e = parse_enum(val, &BLENDOP)?;
        obj.alpha_op = e;
        return Ok(());
    }
};

static VARLIST_PIPELINE: phf::Map<&'static str, VarParseFunc<ast::PipelineStatement>> = phf_map! {
    "DepthEnable" => |obj, val|
    {
        if let tree::Value::Bool(b) = val
        {
            obj.depth_enable = b;
            return Ok(());
        }
        return Err(String::from("[Shader Annotation Language] Expected bool for variable 'DepthEnable'"));
    },
    "DepthWriteEnable" => |obj, val|
    {
        if let tree::Value::Bool(b) = val
        {
            obj.depth_write_enable = b;
            return Ok(());
        }
        return Err(String::from("[Shader Annotation Language] Expected bool for variable 'DepthWriteEnable'"));
    },
    "ScissorEnable" => |obj, val|
    {
        if let tree::Value::Bool(b) = val
        {
            obj.scissor_enable = b;
            return Ok(());
        }
        return Err(String::from("[Shader Annotation Language] Expected bool for variable 'ScissorEnable'"));
    },
    "RenderMode" => |obj, val|
    {
        let e = parse_enum(val, &RENDERMODE)?;
        obj.render_mode = e;
        return Ok(());
    },
    "CullingMode" => |obj, val|
    {
        let e = parse_enum(val, &CULLINGMODE)?;
        obj.culling_mode = e;
        return Ok(());
    }
};

fn parse_varlist<T: ast::VarlistStatement>(
    varlist: tree::VariableList,
    map: &phf::Map<&'static str, VarParseFunc<T>>,
    fallback: Option<VarParseFallback<T>>
) -> Result<T, String>
{
    let mut obj = T::new(varlist.name);

    for v in varlist.vars {
        if let Some(func) = map.get(&*v.name) {
            func(&mut obj, v.value)?;
        } else if let Some(func) = fallback {
            func(&mut obj, &v.name, v.value)?;
        } else {
            return Err(format!("[Shader Annotation Language] Unknown variable name {}", v.name));
        }
    }
    return Ok(obj);
}

fn gen_item(elem: tree::Root, expand_use: bool, module_paths: &Vec<PathBuf>) -> Result<ast::Statement, String>
{
    match elem {
        tree::Root::Constant(c) => {
            let prop = parse_prop(c)?;
            return Ok(ast::Statement::Constant(prop));
        },
        tree::Root::ConstantBuffer(s) => {
            let st = parse_struct(s, "constant buffer")?;
            return Ok(ast::Statement::ConstantBuffer(st));
        },
        tree::Root::Output(c) => {
            let prop = parse_prop(c)?;
            match prop.ptype {
                ast::PropertyType::Sampler => {
                    return Err(String::from(
                        "[Shader Annotation Language] Only vectors and scalars are allowed as render target outputs"
                    ))
                },
                ast::PropertyType::Texture2D(_, _) => {
                    return Err(String::from(
                        "[Shader Annotation Language] Only vectors and scalars are allowed as render target outputs"
                    ))
                },
                ast::PropertyType::Texture3D(_, _) => {
                    return Err(String::from(
                        "[Shader Annotation Language] Only vectors and scalars are allowed as render target outputs"
                    ))
                },
                ast::PropertyType::Texture2DArray(_, _) => {
                    return Err(String::from(
                        "[Shader Annotation Language] Only vectors and scalars are allowed as render target outputs"
                    ))
                },
                ast::PropertyType::TextureCube(_, _) => {
                    return Err(String::from(
                        "[Shader Annotation Language] Only vectors and scalars are allowed as render target outputs"
                    ))
                },
                ast::PropertyType::Matrix(_, _) => {
                    return Err(String::from(
                        "[Shader Annotation Language] Only vectors and scalars are allowed as render target outputs"
                    ))
                },
                _ => ()
            };
            return Ok(ast::Statement::Output(prop));
        },
        tree::Root::VertexFormat(s) => {
            let st = parse_struct(s, "vertex format")?;
            return Ok(ast::Statement::VertexFormat(st));
        },
        tree::Root::Pipeline(v) => {
            let vl = parse_varlist(
                v,
                &VARLIST_PIPELINE,
                Some(|obj, name, val| {
                    if let Some(id) = name.find("::") {
                        let prop = &name[0..id];
                        let output = &name[id + 2..];
                        if prop == "BlendFunc" {
                            if let tree::Value::Identifier(v) = val {
                                obj.blend_functions.insert(String::from(output), String::from(v));
                                return Ok(());
                            }
                            return Err(String::from(
                                "[Shader Annotation Language] Expected identifier for variable 'BlendFunc'"
                            ));
                        } else {
                            return Err(format!("[Shader Annotation Language] Unknown variable name {}", prop));
                        }
                    }
                    return Err(format!("[Shader Annotation Language] Unknown variable name {}", name));
                })
            )?;
            return Ok(ast::Statement::Pipeline(vl));
        },
        tree::Root::Blendfunc(v) => {
            let vl = parse_varlist(v, &VARLIST_BLENDFUNC, None)?;
            return Ok(ast::Statement::Blendfunc(vl));
        },
        tree::Root::Use(mut u) => {
            if !expand_use {
                return Ok(ast::Statement::Noop);
            }
            for p in module_paths {
                u.module.push_str(".sal");
                let file = p.join(&u.module);
                if file.exists() && file.is_file() {
                    let statements = parse_file(&file, false, &Vec::new())?;
                    for stmt in statements {
                        if let Some(name) = stmt.get_name() {
                            if name == u.member {
                                return Ok(stmt);
                            }
                        }
                    }
                    return Err(format!(
                        "[Shader Annotation Language] Could not find member {} in module {}",
                        u.member, u.module
                    ));
                }
            }
        }
    }
    return Ok(ast::Statement::Noop);
}

pub fn build_ast(
    elems: Vec<tree::Root>,
    expand_use: bool,
    module_paths: &Vec<PathBuf>
) -> Result<Vec<ast::Statement>, String>
{
    let mut stvec = Vec::new();

    for v in elems {
        let item = gen_item(v, expand_use, module_paths)?;
        stvec.push(item);
    }
    return Ok(stvec);
}

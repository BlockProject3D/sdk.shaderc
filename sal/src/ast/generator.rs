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

use std::{path::PathBuf, vec::Vec};

use phf::phf_map;
use crate::ast::error::{Error, ValueType};

use crate::ast::tree as ast;
use crate::ast::tree::{TextureType, VectorType};
use crate::parser::tree as tree;
use crate::utils::parse_file;

fn parse_vec_base(t: &str) -> Result<VectorType, Error>
{
    let size = match &t[3..t.len() - 1].parse::<u8>() {
        Err(e) => {
            return Err(Error::VectorSize(e.clone()));
        },
        Ok(v) => *v
    };
    let item = match &t[t.len() - 1..] {
        "f" => ast::BaseType::Float,
        "d" => ast::BaseType::Double,
        "u" => ast::BaseType::Uint,
        "i" => ast::BaseType::Int,
        "b" => ast::BaseType::Bool,
        _ => {
            return Err(Error::UnknownVectorType(t[t.len() - 1..].into()));
        },
    };
    return Ok(VectorType {
        item,
        size
    });
}

fn parse_vec(t: &str) -> Result<VectorType, Error>
{
    if !t.starts_with("vec") {
        return Err(Error::UnknownType(t.into()));
    }
    return parse_vec_base(t);
}

fn try_parse_matrix(t: &str) -> Result<Option<ast::PropertyType>, Error>
{
    if !t.starts_with("mat") {
        return Ok(None);
    }
    let vtype = parse_vec_base(t)?;
    return Ok(Some(ast::PropertyType::Matrix(vtype)));
}

fn try_parse_texture(t: &str, subt: Option<&str>) -> Result<Option<ast::PropertyType>, Error>
{
    if let Some(st) = subt {
        return match t {
            "Texture2D" | "Texture3D" | "Texture2DArray" | "TextureCube" => {
                let ptype = match parse_type(st)? {
                    ast::PropertyType::Scalar(t) => TextureType::Scalar(t),
                    ast::PropertyType::Vector(t) => TextureType::Vector(t),
                    _ => return Err(Error::UnknownTextureType([t, st].join(":")))
                };
                unsafe {
                    match t {
                        "Texture2D" => Ok(Some(ast::PropertyType::Texture2D(ptype))),
                        "Texture3D" => Ok(Some(ast::PropertyType::Texture3D(ptype))),
                        "Texture2DArray" => Ok(Some(ast::PropertyType::Texture2DArray(ptype))),
                        "TextureCube" => Ok(Some(ast::PropertyType::TextureCube(ptype))),
                        _ => std::hint::unreachable_unchecked()
                    }
                }
            },
            _ => Ok(None)
        };
    }
    return Ok(None);
}

fn parse_type(t: &str) -> Result<ast::PropertyType, Error>
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
            let vtype = parse_vec(t)?;
            Ok(ast::PropertyType::Vector(vtype))
        }
    }
}

fn parse_prop(p: tree::Property) -> Result<ast::Property, Error>
{
    let ptype = parse_type(&p.ptype)?;
    return Ok(ast::Property {
        ptype,
        pname: p.pname,
        pattr: p.pattr
    });
}

fn parse_struct(s: tree::Struct) -> Result<ast::Struct, Error>
{
    let mut plist = Vec::new();

    for v in s.props {
        let p = parse_prop(v)?;
        match p.ptype {
            ast::PropertyType::Sampler
            | ast::PropertyType::Texture2D(_)
            | ast::PropertyType::Texture3D(_)
            | ast::PropertyType::Texture2DArray(_)
            | ast::PropertyType::TextureCube(_)
            => return Err(Error::BannedType(p.ptype)),
            _ => ()
        };
        plist.push(p);
    }
    return Ok(ast::Struct {
        name: s.name,
        props: plist
    });
}

static BLENDFACTOR: phf::Map<&'static str, ast::BlendFactor> = phf_map! {
    "Zero" => ast::BlendFactor::Zero,
    "One" => ast::BlendFactor::One,
    "SrcColor" => ast::BlendFactor::SrcColor,
    "OneMinusSrcColor" => ast::BlendFactor::OneMinusSrcColor,
    "SrcAlpha" => ast::BlendFactor::SrcAlpha,
    "OneMinusSrcAlpha" => ast::BlendFactor::OneMinusSrcAlpha,
    "DstColor" => ast::BlendFactor::DstColor,
    "OneMinusDstColor" => ast::BlendFactor::OneMinusDstColor,
    "DstAlpha" => ast::BlendFactor::DstAlpha,
    "OneMinusDstAlpha" => ast::BlendFactor::OneMinusDstAlpha,
    "SrcAlphaSaturate" => ast::BlendFactor::SrcAlphaSaturate,
    "Src1Color" => ast::BlendFactor::Src1Color,
    "OneMinusSrc1Color" => ast::BlendFactor::OneMinusSrc1Color,
    "Src1Alpha" => ast::BlendFactor::Src1Alpha,
    "OneMinusSrc1Alpha" => ast::BlendFactor::OneMinusSrc1Alpha
};

static BLENDOP: phf::Map<&'static str, ast::BlendOperator> = phf_map! {
    "Add" => ast::BlendOperator::Add,
    "Sub" => ast::BlendOperator::Subtract,
    "InvSub" => ast::BlendOperator::InverseSubtract,
    "Min" => ast::BlendOperator::Min,
    "Max" => ast::BlendOperator::Max
};

static RENDERMODE: phf::Map<&'static str, ast::RenderMode> = phf_map! {
    "Triangles" => ast::RenderMode::Triangles,
    "Wireframe" => ast::RenderMode::Wireframe,
    "Patches" => ast::RenderMode::Patches
};

static CULLINGMODE: phf::Map<&'static str, ast::CullingMode> = phf_map! {
    "FrontFace" => ast::CullingMode::FrontFace,
    "BackFace" => ast::CullingMode::BackFace,
    "Disabled" => ast::CullingMode::Disabled
};

fn parse_enum<T: Copy>(value: tree::Value, map: &phf::Map<&'static str, T>) -> Result<T, Error>
{
    if let tree::Value::Identifier(id) = value {
        if let Some(e) = map.get(&*id) {
            return Ok(*e);
        }
        return Err(Error::UnknownEnum(id));
    }
    return Err(Error::UnexpectedType {
        expected: ValueType::Enum,
        actual: value
    });
}

fn parse_bool(value: tree::Value) -> Result<bool, Error>
{
    if let tree::Value::Bool(b) = value {
        Ok(b)
    } else {
        Err(Error::UnexpectedType {
            expected: ValueType::Bool,
            actual: value
        })
    }
}

type VarParseFunc<T> = fn(obj: &mut T, value: tree::Value) -> Result<(), Error>;

static VARLIST_BLENDFUNC: phf::Map<&'static str, VarParseFunc<ast::BlendfuncStatement>> = phf_map! {
    "SrcColor" => |obj, val|
    {
        obj.src_color = parse_enum(val, &BLENDFACTOR)?;
        return Ok(());
    },
    "DstColor" => |obj, val|
    {
        obj.dst_color = parse_enum(val, &BLENDFACTOR)?;
        return Ok(());
    },
    "SrcAlpha" => |obj, val|
    {
        obj.src_alpha = parse_enum(val, &BLENDFACTOR)?;
        return Ok(());
    },
    "DstAlpha" => |obj, val|
    {
        obj.dst_alpha = parse_enum(val, &BLENDFACTOR)?;
        return Ok(());
    },
    "ColorOp" => |obj, val|
    {
        obj.color_op = parse_enum(val, &BLENDOP)?;
        return Ok(());
    },
    "AlphaOp" => |obj, val|
    {
        obj.alpha_op = parse_enum(val, &BLENDOP)?;
        return Ok(());
    }
};

static VARLIST_PIPELINE: phf::Map<&'static str, VarParseFunc<ast::PipelineStatement>> = phf_map! {
    "DepthEnable" => |obj, val|
    {
        obj.depth_enable = parse_bool(val)?;
        Ok(())
    },
    "DepthWriteEnable" => |obj, val|
    {
        obj.depth_write_enable = parse_bool(val)?;
        Ok(())
    },
    "ScissorEnable" => |obj, val|
    {
        obj.scissor_enable = parse_bool(val)?;
        Ok(())
    },
    "RenderMode" => |obj, val|
    {
        obj.render_mode = parse_enum(val, &RENDERMODE)?;
        return Ok(());
    },
    "CullingMode" => |obj, val|
    {
        obj.culling_mode = parse_enum(val, &CULLINGMODE)?;
        return Ok(());
    }
};

fn parse_varlist<T: ast::VarlistStatement>(varlist: tree::VariableList, map: &phf::Map<&'static str, VarParseFunc<T>>) -> Result<T, Error>
{
    let mut obj = T::new(varlist.name);

    for v in varlist.vars {
        if let Some(func) = map.get(&*v.name) {
            func(&mut obj, v.value)?;
        } else {
            return Err(Error::UnknownVariable(v.name));
        }
    }
    return Ok(obj);
}

fn gen_item(elem: tree::Root, expand_use: bool, module_paths: &Vec<PathBuf>) -> Result<ast::Statement, Error>
{
    match elem {
        tree::Root::Constant(c) => {
            let prop = parse_prop(c)?;
            return Ok(ast::Statement::Constant(prop));
        },
        tree::Root::ConstantBuffer(s) => {
            let st = parse_struct(s)?;
            return Ok(ast::Statement::ConstantBuffer(st));
        },
        tree::Root::Output(c) => {
            let prop = parse_prop(c)?;
            match prop.ptype {
                ast::PropertyType::Sampler
                | ast::PropertyType::Texture2D(_)
                | ast::PropertyType::Texture3D(_)
                | ast::PropertyType::Texture2DArray(_)
                | ast::PropertyType::TextureCube(_)
                | ast::PropertyType::Matrix(_)
                => return Err(Error::BannedType(prop.ptype)),
                _ => ()
            };
            return Ok(ast::Statement::Output(prop));
        },
        tree::Root::VertexFormat(s) => {
            let st = parse_struct(s)?;
            return Ok(ast::Statement::VertexFormat(st));
        },
        tree::Root::Pipeline(v) => {
            let vl = parse_varlist(v, &VARLIST_PIPELINE)?;
            return Ok(ast::Statement::Pipeline(vl));
        },
        tree::Root::Blendfunc(v) => {
            let vl = parse_varlist(v, &VARLIST_BLENDFUNC)?;
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
                    let statements = parse_file(&file, false, &Vec::new()).unwrap(); //TODO: fix
                    for stmt in statements {
                        if let Some(name) = stmt.get_name() {
                            if name == u.member {
                                return Ok(stmt);
                            }
                        }
                    }
                    return Err(Error::UseNotFound(u));
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
) -> Result<Vec<ast::Statement>, Error>
{
    let mut stvec = Vec::new();

    for v in elems {
        let item = gen_item(v, expand_use, module_paths)?;
        stvec.push(item);
    }
    return Ok(stvec);
}

#[cfg(test)]
mod tests
{
    use std::collections::HashMap;
    use crate::{Lexer, Parser};
    use crate::ast::tree::{BaseType, BlendFactor, BlendfuncStatement, BlendOperator, CullingMode, PipelineStatement, Property, PropertyType, RenderMode, Statement, Struct};
    use super::*;

    #[test]
    fn basic_ast()
    {
        let source_code = b"
            const float DeltaTime;
            const uint FrameCount;
            const mat3f ModelViewMatrix;
            const mat3f ProjectionMatrix;
            const struct PerMaterial
            {
                vec4f BaseColor;
                float UvMultiplier;
            }
        ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        let mut parser = Parser::new(lexer);
        let roots = parser.parse().unwrap();
        let incs = Vec::new();
        let ast = build_ast(roots, false, &incs).unwrap();
        let expected_ast = vec![
            Statement::Constant(Property {
                pname: "DeltaTime".into(),
                ptype: PropertyType::Scalar(BaseType::Float),
                pattr: None
            }),
            Statement::Constant(Property {
                pname: "FrameCount".into(),
                ptype: PropertyType::Scalar(BaseType::Uint),
                pattr: None
            }),
            Statement::Constant(Property {
                pname: "ModelViewMatrix".into(),
                ptype: PropertyType::Matrix(VectorType {
                    item: BaseType::Float,
                    size: 3
                }),
                pattr: None
            }),
            Statement::Constant(Property {
                pname: "ProjectionMatrix".into(),
                ptype: PropertyType::Matrix(VectorType {
                    item: BaseType::Float,
                    size: 3
                }),
                pattr: None
            }),
            Statement::ConstantBuffer(Struct {
                name: "PerMaterial".into(),
                props: vec![
                    Property {
                        pname: "BaseColor".into(),
                        ptype: PropertyType::Vector(VectorType {
                            item: BaseType::Float,
                            size: 4
                        }),
                        pattr: None
                    },
                    Property {
                        pname: "UvMultiplier".into(),
                        ptype: PropertyType::Scalar(BaseType::Float),
                        pattr: None
                    }
                ]
            })
        ];
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn complex_ast()
    {
        let source_code = b"
            const Sampler BaseSampler;
            const Texture2D:vec4f BaseTexture : BaseSampler;
            const Texture2D:float NoiseTexture : BaseSampler;
            const struct PerMaterial
            {
                vec4f BaseColor;
                float Specular : Pack;
                float UvMultiplier : Pack;
            }
        ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        let mut parser = Parser::new(lexer);
        let roots = parser.parse().unwrap();
        let incs = Vec::new();
        let ast = build_ast(roots, false, &incs).unwrap();
        let expected_ast = vec![
            Statement::Constant(Property {
                pname: "BaseSampler".into(),
                ptype: PropertyType::Sampler,
                pattr: None
            }),
            Statement::Constant(Property {
                pname: "BaseTexture".into(),
                ptype: PropertyType::Texture2D(TextureType::Vector(VectorType {
                    item: BaseType::Float,
                    size: 4
                })),
                pattr: Some("BaseSampler".into())
            }),
            Statement::Constant(Property {
                pname: "NoiseTexture".into(),
                ptype: PropertyType::Texture2D(TextureType::Scalar(BaseType::Float)),
                pattr: Some("BaseSampler".into())
            }),
            Statement::ConstantBuffer(Struct {
                name: "PerMaterial".into(),
                props: vec![
                    Property {
                        pname: "BaseColor".into(),
                        ptype: PropertyType::Vector(VectorType {
                            item: BaseType::Float,
                            size: 4
                        }),
                        pattr: None
                    },
                    Property {
                        pname: "Specular".into(),
                        ptype: PropertyType::Scalar(BaseType::Float),
                        pattr: Some("Pack".into())
                    },
                    Property {
                        pname: "UvMultiplier".into(),
                        ptype: PropertyType::Scalar(BaseType::Float),
                        pattr: Some("Pack".into())
                    }
                ]
            })
        ];
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn basic_output()
    {
        let source_code = b"
            output vec4f FragColor;
        ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        let mut parser = Parser::new(lexer);
        let roots = parser.parse().unwrap();
        let incs = Vec::new();
        let ast = build_ast(roots, false, &incs).unwrap();
        let expected_ast = vec![
            Statement::Output(Property {
                pname: "FragColor".into(),
                ptype: PropertyType::Vector(VectorType {
                    item: BaseType::Float,
                    size: 4
                }),
                pattr: None
            })
        ];
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn basic_vformat()
    {
        let source_code = b"
            vformat struct Vertex
            {
                vec3f Pos;
            }
        ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        let mut parser = Parser::new(lexer);
        let roots = parser.parse().unwrap();
        let incs = Vec::new();
        let ast = build_ast(roots, false, &incs).unwrap();
        let expected_ast = vec![
            Statement::VertexFormat(Struct {
                name: "Vertex".into(),
                props: vec![
                    Property {
                        pname: "Pos".into(),
                        ptype: PropertyType::Vector(VectorType {
                            item: BaseType::Float,
                            size: 3
                        }),
                        pattr: None
                    }
                ]
            })
        ];
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn basic_pipeline()
    {
        let source_code = b"
            pipeline Test
            {
                DepthEnable = true;
                DepthWriteEnable = true;
                ScissorEnable = false;
                RenderMode = Triangles;
                CullingMode = BackFace;
            }
        ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        let mut parser = Parser::new(lexer);
        let roots = parser.parse().unwrap();
        let incs = Vec::new();
        let ast = build_ast(roots, false, &incs).unwrap();
        let expected_ast = vec![
            Statement::Pipeline(PipelineStatement {
                name: "Test".into(),
                depth_enable: true,
                depth_write_enable: true,
                scissor_enable: false,
                render_mode: RenderMode::Triangles,
                culling_mode: CullingMode::BackFace
            })
        ];
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn blendfunc_output()
    {
        let source_code = b"
            output vec4f FragColor;

            blendfunc FragColor
            {
                SrcColor = SrcAlpha;
                DstColor = OneMinusSrcAlpha;
                SrcAlpha = SrcAlpha;
                DstAlpha = OneMinusSrcAlpha;
                ColorOp = Add;
                AlphaOp = Add;
            }
        ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        let mut parser = Parser::new(lexer);
        let roots = parser.parse().unwrap();
        let incs = Vec::new();
        let ast = build_ast(roots, false, &incs).unwrap();
        let expected_ast = vec![
            Statement::Output(Property {
                pname: "FragColor".into(),
                ptype: PropertyType::Vector(VectorType {
                    item: BaseType::Float,
                    size: 4
                }),
                pattr: None
            }),
            Statement::Blendfunc(BlendfuncStatement {
                name: "FragColor".into(),
                src_color: BlendFactor::SrcAlpha,
                dst_color: BlendFactor::OneMinusSrcAlpha,
                src_alpha: BlendFactor::SrcAlpha,
                dst_alpha: BlendFactor::OneMinusSrcAlpha,
                color_op: BlendOperator::Add,
                alpha_op: BlendOperator::Add
            })
        ];
        assert_eq!(ast, expected_ast);
    }
}

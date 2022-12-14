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

use phf::phf_map;

use crate::{
    ast::{
        error::{Error, TypeError, ValueError, ValueType},
        tree as ast
    },
    parser::tree
};
use crate::ast::tree::ArrayType;
use crate::ast::{RefResolver, Visitor};
use crate::parser::tree::{Property, Struct, Use, VariableList};

fn parse_vec_base<T>(ptype: &str) -> Result<ast::VectorType, TypeError<T>>
{
    let size = match &ptype[3..ptype.len() - 1].parse::<u8>() {
        Err(e) => {
            return Err(TypeError::VectorSize(e.clone()));
        },
        Ok(v) => *v
    };
    let item = match &ptype[ptype.len() - 1..] {
        "f" => ast::BaseType::Float,
        "d" => ast::BaseType::Double,
        "u" => ast::BaseType::Uint,
        "i" => ast::BaseType::Int,
        "b" => ast::BaseType::Bool,
        _ => {
            return Err(TypeError::UnknownVector(ptype[ptype.len() - 1..].into()));
        }
    };
    Ok(ast::VectorType { item, size })
}

fn try_parse_vec<T>(ptype: &str) -> Result<Option<ast::PropertyType<T>>, TypeError<T>>
{
    if !ptype.starts_with("vec") {
        return Ok(None);
    }
    let vtype = parse_vec_base(ptype)?;
    Ok(Some(ast::PropertyType::Vector(vtype)))
}

fn try_parse_matrix<T>(ptype: &str) -> Result<Option<ast::PropertyType<T>>, TypeError<T>>
{
    if !ptype.starts_with("mat") {
        return Ok(None);
    }
    let vtype = parse_vec_base(ptype)?;
    Ok(Some(ast::PropertyType::Matrix(vtype)))
}

fn try_parse_texture<A: RefResolver>(ptype: &str, ptype_attr: Option<&str>, ast: &A) -> Result<Option<ast::PropertyType<A::Key>>, TypeError<A::Key>>
{
    if let Some(subtype) = ptype_attr {
        return match ptype {
            "Texture2D" | "Texture3D" | "Texture2DArray" | "TextureCube" => {
                let ttype = match parse_type(subtype, None, None, ast)? {
                    ast::PropertyType::Scalar(t) => ast::TextureType::Scalar(t),
                    ast::PropertyType::Vector(t) => ast::TextureType::Vector(t),
                    _ => return Err(TypeError::UnknownTexture([ptype, subtype].join(":")))
                };
                unsafe {
                    match ptype {
                        "Texture2D" => Ok(Some(ast::PropertyType::Texture2D(ttype))),
                        "Texture3D" => Ok(Some(ast::PropertyType::Texture3D(ttype))),
                        "Texture2DArray" => Ok(Some(ast::PropertyType::Texture2DArray(ttype))),
                        "TextureCube" => Ok(Some(ast::PropertyType::TextureCube(ttype))),
                        _ => std::hint::unreachable_unchecked()
                    }
                }
            },
            _ => Ok(None)
        };
    }
    Ok(None)
}

fn try_parse_array<A: RefResolver>(ptype: &str, ptype_arr: Option<u32>, ast: &A) -> Result<Option<ast::PropertyType<A::Key>>, TypeError<A::Key>>
{
    if let Some(size) = ptype_arr {
        let item = match parse_type(ptype, None, None, ast)? {
            ast::PropertyType::Vector(t) => ast::ArrayItemType::Vector(t),
            ast::PropertyType::Matrix(t) => ast::ArrayItemType::Matrix(t),
            ast::PropertyType::StructRef(t) => ast::ArrayItemType::StructRef(t),
            _ => return Err(TypeError::Unknown(ptype.into()))
        };
        Ok(Some(ast::PropertyType::Array(ArrayType {
            item,
            size
        })))
    } else {
        Ok(None)
    }
}

fn parse_type<A: RefResolver>(ptype: &str, ptype_arr: Option<u32>, ptype_attr: Option<&str>, ast: &A) -> Result<ast::PropertyType<A::Key>, TypeError<A::Key>>
{
    match ptype {
        "Sampler" => Ok(ast::PropertyType::Sampler),
        "float" => Ok(ast::PropertyType::Scalar(ast::BaseType::Float)),
        "double" => Ok(ast::PropertyType::Scalar(ast::BaseType::Double)),
        "int" => Ok(ast::PropertyType::Scalar(ast::BaseType::Int)),
        "uint" => Ok(ast::PropertyType::Scalar(ast::BaseType::Uint)),
        "bool" => Ok(ast::PropertyType::Scalar(ast::BaseType::Bool)),
        _ => {
            if let Some(elem) = try_parse_array(ptype, ptype_arr, ast)? {
                return Ok(elem)
            }
            if let Some(elem) = try_parse_matrix(ptype)? {
                return Ok(elem);
            }
            if let Some(elem) = try_parse_texture(ptype, ptype_attr, ast)? {
                return Ok(elem);
            }
            if let Some(elem) = try_parse_vec(ptype)? {
                return Ok(elem);
            }
            let val = ast.resolve_struct_ref(ptype)
                .ok_or_else(|| TypeError::Unknown(ptype.into()))?;
            Ok(ast::PropertyType::StructRef(val))
        }
    }
}

fn parse_attribute<T>(pattr: Option<String>) -> Result<Option<ast::Attribute>, TypeError<T>>
{
    if pattr.is_none() {
        return Ok(None);
    }
    let val = pattr.unwrap();
    if val == "Pack" {
        return Ok(Some(ast::Attribute::Pack));
    }
    if val.starts_with("ORDER_") {
        let order = &val[6..].parse::<u32>().map_err(|e| TypeError::AttributeOrder(e))?;
        Ok(Some(ast::Attribute::Order(*order)))
    } else {
        Ok(Some(ast::Attribute::Identifier(val)))
    }
}

fn parse_prop<A: RefResolver>(p: tree::Property, ast: &A) -> Result<ast::Property<A::Key>, TypeError<A::Key>>
{
    let ptype = parse_type(&p.ptype, p.ptype_arr, p.ptype_attr.as_deref(), ast)?;
    Ok(ast::Property {
        ptype,
        pname: p.pname,
        pattr: parse_attribute(p.pattr)?
    })
}

fn parse_struct<A: RefResolver, F: Fn(&ast::PropertyType<A::Key>) -> bool>(s: tree::Struct, is_further_banned: F, ast: &A) -> Result<ast::Struct<A::Key>, TypeError<A::Key>>
{
    let mut plist = Vec::new();

    for v in s.props {
        let p = parse_prop(v, ast)?;
        match p.ptype {
            ast::PropertyType::Sampler
            | ast::PropertyType::Texture2D(_)
            | ast::PropertyType::Texture3D(_)
            | ast::PropertyType::Texture2DArray(_)
            | ast::PropertyType::TextureCube(_) => return Err(TypeError::Banned(p.ptype)),
            _ => ()
        };
        if is_further_banned(&p.ptype) {
            return Err(TypeError::Banned(p.ptype));
        }
        plist.push(p);
    }
    Ok(ast::Struct {
        name: s.name,
        attr: parse_attribute(s.attr)?,
        props: plist
    })
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

fn parse_enum<T: Copy>(value: tree::Value, map: &phf::Map<&'static str, T>) -> Result<T, ValueError>
{
    if let tree::Value::Identifier(id) = value {
        if let Some(e) = map.get(&*id) {
            return Ok(*e);
        }
        return Err(ValueError::UnknownEnum(id));
    }
    Err(ValueError::Unexpected {
        expected: ValueType::Enum,
        actual: value
    })
}

fn parse_bool(value: tree::Value) -> Result<bool, ValueError>
{
    if let tree::Value::Bool(b) = value {
        Ok(b)
    } else {
        Err(ValueError::Unexpected {
            expected: ValueType::Bool,
            actual: value
        })
    }
}

type VarParseFunc<T> = fn(obj: &mut T, value: tree::Value) -> Result<(), ValueError>;

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

fn parse_varlist<T: ast::VarlistStatement>(
    varlist: tree::VariableList,
    map: &phf::Map<&'static str, VarParseFunc<T>>
) -> Result<T, ValueError>
{
    let mut obj = T::new(varlist.name);

    for v in varlist.vars {
        if let Some(func) = map.get(&*v.name) {
            func(&mut obj, v.value)?;
        } else {
            return Err(ValueError::UnknownVariable(v.name));
        }
    }
    Ok(obj)
}

pub struct AstBuilder<V, A>
{
    //statements: Vec<Statement>,
    visitor: V,
    ast: A
}

impl<A: RefResolver, V: Visitor<A>> AstBuilder<V, A>
{
    pub fn new(ast: A, visitor: V) -> AstBuilder<V, A>
    {
        AstBuilder {
            visitor,
            ast
        }
    }

    pub fn into_inner(self) -> A
    {
        self.ast
    }
}

impl<A: RefResolver, V: Visitor<A>> crate::parser::Visitor for AstBuilder<V, A>
{
    type Error = Error<A::Key, V::Error>;

    fn visit_constant(&mut self, val: Property) -> Result<(), Self::Error> {
        let prop = parse_prop(val, &self.ast)?;
        self.visitor.visit_constant(&mut self.ast, prop).map_err(Error::Visitor)?;
        Ok(())
    }

    fn visit_constant_buffer(&mut self, val: Struct) -> Result<(), Self::Error> {
        let st = parse_struct(val, |_| false, &self.ast)?;
        self.visitor.visit_constant_buffer(&mut self.ast, st).map_err(Error::Visitor)?;
        Ok(())
    }

    fn visit_output(&mut self, val: Property) -> Result<(), Self::Error> {
        let prop = parse_prop(val, &self.ast)?;
        match prop.ptype {
            ast::PropertyType::Sampler
            | ast::PropertyType::Texture2D(_)
            | ast::PropertyType::Texture3D(_)
            | ast::PropertyType::Texture2DArray(_)
            | ast::PropertyType::TextureCube(_)
            | ast::PropertyType::Matrix(_) => return Err(Error::Type(TypeError::Banned(prop.ptype))),
            _ => ()
        };
        self.visitor.visit_output(&mut self.ast, prop).map_err(Error::Visitor)?;
        Ok(())
    }

    fn visit_vertex_format(&mut self, val: Struct) -> Result<(), Self::Error> {
        let st = parse_struct(val, |v| {
            match v {
                ast::PropertyType::Matrix(_) |
                ast::PropertyType::Vector(_) |
                ast::PropertyType::Scalar(_) => false,
                _ => true
            }
        }, &self.ast)?;
        self.visitor.visit_vertex_format(&mut self.ast, st).map_err(Error::Visitor)?;
        Ok(())
    }

    fn visit_use(&mut self, val: Use) -> Result<(), Self::Error> {
        self.visitor.visit_use(&mut self.ast, val.module, val.member).map_err(Error::Visitor)?;
        Ok(())
    }

    fn visit_pipeline(&mut self, val: VariableList) -> Result<(), Self::Error> {
        let vl = parse_varlist(val, &VARLIST_PIPELINE)?;
        self.visitor.visit_pipeline(&mut self.ast , vl).map_err(Error::Visitor)?;
        Ok(())
    }

    fn visit_blendfunc(&mut self, val: VariableList) -> Result<(), Self::Error> {
        let vl = parse_varlist(val, &VARLIST_BLENDFUNC)?;
        self.visitor.visit_blendfunc(&mut self.ast, vl).map_err(Error::Visitor)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{
        ast::{
            tree::{
                BaseType,
                BlendFactor,
                BlendOperator,
                BlendfuncStatement,
                CullingMode,
                PipelineStatement,
                Property,
                PropertyType,
                RenderMode,
                Statement,
                Struct,
                TextureType,
                VectorType
            },
        },
        lexer::Lexer,
        parser::Parser
    };
    use crate::ast::RefResolver;
    use crate::ast::tree::{ArrayItemType, Attribute};

    struct VecVisitor {}

    impl RefResolver for Vec<Statement> {
        type Key = String;

        fn resolve_struct_ref(&self, name: &str) -> Option<Self::Key> {
            Some(name.into())
        }
    }

    impl Visitor<Vec<Statement>> for VecVisitor {
        type Error = ();

        fn visit_constant(&mut self, ast: &mut Vec<Statement>, val: Property) -> Result<(), Self::Error> {
            ast.push(Statement::Constant(val));
            Ok(())
        }

        fn visit_output(&mut self, ast: &mut Vec<Statement>, val: Property) -> Result<(), Self::Error> {
            ast.push(Statement::Output(val));
            Ok(())
        }

        fn visit_constant_buffer(&mut self, ast: &mut Vec<Statement>, val: Struct) -> Result<(), Self::Error> {
            ast.push(Statement::ConstantBuffer(val));
            Ok(())
        }

        fn visit_vertex_format(&mut self, ast: &mut Vec<Statement>, val: Struct) -> Result<(), Self::Error> {
            ast.push(Statement::VertexFormat(val));
            Ok(())
        }

        fn visit_pipeline(&mut self, ast: &mut Vec<Statement>, val: PipelineStatement) -> Result<(), Self::Error> {
            ast.push(Statement::Pipeline(val));
            Ok(())
        }

        fn visit_blendfunc(&mut self, ast: &mut Vec<Statement>, val: BlendfuncStatement) -> Result<(), Self::Error> {
            ast.push(Statement::Blendfunc(val));
            Ok(())
        }

        fn visit_noop(&mut self, ast: &mut Vec<Statement>) -> Result<(), Self::Error> {
            ast.push(Statement::Noop);
            Ok(())
        }

        fn visit_use(&mut self, ast: &mut Vec<Statement>, _: String, _: String) -> Result<(), Self::Error> {
            self.visit_noop(ast)
        }
    }

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
        let ast = parser.parse(AstBuilder::new(Vec::new(), VecVisitor {})).unwrap().into_inner();
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
                attr: None,
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
                    },
                ]
            }),
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
            const struct PerMaterial : ORDER_1
            {
                vec4f BaseColor;
                float Specular : Pack;
                float UvMultiplier : Pack;
            }
        ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        let mut parser = Parser::new(lexer);
        let ast = parser.parse(AstBuilder::new(Vec::new(), VecVisitor {})).unwrap().into_inner();
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
                pattr: Some(Attribute::Identifier("BaseSampler".into()))
            }),
            Statement::Constant(Property {
                pname: "NoiseTexture".into(),
                ptype: PropertyType::Texture2D(TextureType::Scalar(BaseType::Float)),
                pattr: Some(Attribute::Identifier("BaseSampler".into()))
            }),
            Statement::ConstantBuffer(Struct {
                name: "PerMaterial".into(),
                attr: Some(Attribute::Order(1)),
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
                        pattr: Some(Attribute::Pack)
                    },
                    Property {
                        pname: "UvMultiplier".into(),
                        ptype: PropertyType::Scalar(BaseType::Float),
                        pattr: Some(Attribute::Pack)
                    },
                ]
            }),
        ];
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn ast_arrays()
    {
        let source_code = b"
            const struct Light : Pack { vec4f color; float attenuation; }
            const struct Lighting { uint count; Light[32] lights; }
        ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        let mut parser = Parser::new(lexer);
        let ast = parser.parse(AstBuilder::new(Vec::new(), VecVisitor {})).unwrap().into_inner();
        let expected_ast = vec![
            Statement::ConstantBuffer(Struct {
                name: "Light".into(),
                attr: Some(Attribute::Pack),
                props: vec![
                    Property {
                        pname: "color".into(),
                        ptype: PropertyType::Vector(VectorType {
                            size: 4,
                            item: BaseType::Float
                        }),
                        pattr: None
                    },
                    Property {
                        pname: "attenuation".into(),
                        ptype: PropertyType::Scalar(BaseType::Float),
                        pattr: None
                    }
                ]
            }),
            Statement::ConstantBuffer(Struct {
                name: "Lighting".into(),
                attr: None,
                props: vec![
                    Property {
                        pname: "count".into(),
                        ptype: PropertyType::Scalar(BaseType::Uint),
                        pattr: None
                    },
                    Property {
                        pname: "lights".into(),
                        ptype: PropertyType::Array(ArrayType {
                            size: 32,
                            item: ArrayItemType::StructRef("Light".into())
                        }),
                        pattr: None,
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
        let ast = parser.parse(AstBuilder::new(Vec::new(), VecVisitor {})).unwrap().into_inner();
        let expected_ast = vec![Statement::Output(Property {
            pname: "FragColor".into(),
            ptype: PropertyType::Vector(VectorType {
                item: BaseType::Float,
                size: 4
            }),
            pattr: None
        })];
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
        let ast = parser.parse(AstBuilder::new(Vec::new(), VecVisitor {})).unwrap().into_inner();
        let expected_ast = vec![Statement::VertexFormat(Struct {
            name: "Vertex".into(),
            attr: None,
            props: vec![Property {
                pname: "Pos".into(),
                ptype: PropertyType::Vector(VectorType {
                    item: BaseType::Float,
                    size: 3
                }),
                pattr: None
            }]
        })];
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
        let ast = parser.parse(AstBuilder::new(Vec::new(), VecVisitor {})).unwrap().into_inner();
        let expected_ast = vec![Statement::Pipeline(PipelineStatement {
            name: "Test".into(),
            depth_enable: true,
            depth_write_enable: true,
            scissor_enable: false,
            render_mode: RenderMode::Triangles,
            culling_mode: CullingMode::BackFace
        })];
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
        let ast = parser.parse(AstBuilder::new(Vec::new(), VecVisitor {})).unwrap().into_inner();
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
            }),
        ];
        assert_eq!(ast, expected_ast);
    }
}

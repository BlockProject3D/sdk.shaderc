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
use bp3d_sal::ast::tree::{BaseType, VectorType};
use crate::{FromBpx, Refs, ToBpx};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ArrayItemType
{
    Vector(VectorType),
    Matrix(VectorType),
    StructRef(u16), //Index of referenced symbol in symbol table.
}

#[derive(Copy, Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub struct PropObject
{
    pub name: String,
    pub offset: u32,
    pub ty: PropType
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StructObject
{
    pub size: u32,
    pub props: Vec<PropObject>
}

impl ToBpx for StructObject {}
impl FromBpx for StructObject {}

fn erase_refs(obj: &PropObject) -> PropObject {
    let mut res = obj.clone();
    res.ty = match res.ty {
        PropType::StructRef(_) => PropType::StructRef(0),
        PropType::Array { size, ty } => match ty {
            ArrayItemType::StructRef(_) => PropType::Array {
                size,
                ty: ArrayItemType::StructRef(0)
            },
            _ => PropType::Array { size, ty }
        },
        _ => res.ty
    };
    res
}

impl StructObject {
    fn refs_iter(&self) -> impl Iterator<Item = u16> + '_ {
        self.props.iter().filter_map(|v| match v.ty {
            PropType::StructRef(v) => Some(v),
            PropType::Array { ty, .. } => match ty {
                ArrayItemType::StructRef(v) => Some(v),
                _ => None
            },
            _ => None
        })
    }
}

impl Refs for StructObject {
    // Code duplication required; cannot be fixed; impl Trait is now broken!
    // Now causes "captures lifetime that does not appear in bounds".
    // However it used to work in the past!
    fn list_refs(&self) -> Vec<usize> {
        self.refs_iter().map(|v| v.into()).collect()
    }

    // Code duplication required; cannot be fixed; impl Trait is now broken!
    // Now causes "captures lifetime that does not appear in bounds".
    // However it used to work in the past!
    fn has_refs(&self) -> bool {
        self.refs_iter().count() > 0
    }

    fn clone_erase_refs(&self) -> Self {
        StructObject {
            size: self.size,
            props: self.props.iter().map(|v| erase_refs(v)).collect()
        }
    }
}

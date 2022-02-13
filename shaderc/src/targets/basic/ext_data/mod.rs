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
use bpx::sd::serde::EnumSize;
use bpx::shader::ShaderPack;
use serde::Serialize;

pub trait ToObject<T = ()> where Self: Sized
{
    type Object: Serialize;
    type Context;

    fn to_object(self, ctx: &Self::Context) -> Option<Self::Object>;

    fn to_bpx_object(self, debug: bool, ctx: &Self::Context) -> Result<Option<bpx::sd::Object>, bpx::sd::serde::Error>
    {
        let obj = self.to_object(ctx);
        let res = obj.map(|v| v.serialize(bpx::sd::serde::Serializer::new(EnumSize::U8, debug)));
        let res = match res {
            Some(v) => Some(v?.try_into().unwrap()),
            None => None
        };
        Ok(res)
    }
}

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

    pub fn write(&mut self, builder: bpx::shader::symbol::Builder) -> Result<(), bpx::shader::error::WriteError> {
        let s = builder.build();
        self.map.insert(s.name.clone(), self.inner.get_symbol_count());
        self.inner.add_symbol(s)?;
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

mod objects;
mod structs;

pub use objects::*;
pub use structs::*;

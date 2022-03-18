// Copyright (c) 2022, BlockProject 3D
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
use std::sync::Arc;
use bpx::shader::symbol::{FLAG_EXTERNAL, FLAG_INTERNAL, Type};
use log::debug;
use crate::ext_data::ExtDataPtr;

pub struct Symbol {
    name: Arc<str>,
    index: usize,
    info: bpx::shader::symbol::Symbol,
    ext_data: Option<ExtDataPtr>,
    signature: Option<[u8; 64]>
}

impl Symbol {
    pub fn new(name: String, index: usize, info: bpx::shader::symbol::Symbol, ext_data: Option<ExtDataPtr>) -> Symbol {
        Symbol {
            name: name.into(),
            index,
            info,
            ext_data,
            signature: None
        }
    }

    pub fn signature(&self) -> Option<&[u8; 64]> {
        self.signature.as_ref()
    }

    pub fn set_signature(&mut self, sig: [u8; 64]) {
        self.signature = Some(sig);
    }

    pub fn get_coded_info(&self) -> (u8, u8) {
        let tcode = match self.info.ty {
            Type::Texture => 0,
            Type::Sampler => 1,
            Type::ConstantBuffer => 2,
            Type::Constant => 3,
            Type::VertexFormat => 4,
            Type::Pipeline => 5,
            Type::Output => 6
        };
        (tcode, self.info.register)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn info(&self) -> &bpx::shader::symbol::Symbol {
        &self.info
    }

    pub fn info_mut(&mut self) -> &mut bpx::shader::symbol::Symbol {
        &mut self.info
    }

    pub fn ext_data(&self) -> Option<&ExtDataPtr> {
        self.ext_data.as_ref()
    }
}

pub struct SymbolTree {
    symbols: Vec<Symbol>,
    by_name: HashMap<Arc<str>, usize>,
    by_index: HashMap<usize, usize>
}

impl SymbolTree {
    pub fn empty() -> SymbolTree {
        SymbolTree {
            symbols: Vec::new(),
            by_name: HashMap::new(),
            by_index: HashMap::new()
        }
    }

    pub fn insert(&mut self, sym: Symbol) {
        let new_id = self.symbols.len();
        self.by_name.insert(sym.name.clone(), new_id);
        self.by_index.insert(sym.index, new_id);
        self.symbols.push(sym);
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Symbol> + '_ {
        self.symbols.iter_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Symbol> + '_ {
        self.symbols.iter()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Symbol> {
        self.by_name.get(name).map(|v| &self.symbols[*v])
    }

    //Mass set FLAG_INTERNAL on all symbols.
    pub fn mass_set_internal(&mut self) {
        self.iter_mut().for_each(|v| {
            debug!("Set internal flag for '{}'", v.name());
            v.info_mut().flags &= !FLAG_EXTERNAL;
            v.info_mut().flags |= FLAG_INTERNAL;
        })
    }

    //Offset the references back to 0 in this tree to prepare saving into BPX symbol table section.
    pub fn align_references(&mut self) {
        let map: HashMap<usize, usize> = self.symbols.iter()
            .enumerate()
            .map(|(new_id, v)| (v.index, new_id))
            .collect();
        self.symbols.iter_mut().for_each(|v| {
            v.ext_data = v.ext_data.take().map(|v1| v1.rewrite_refs(&map));
        });
    }
}

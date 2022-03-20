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

// SAL AST structure.

use std::collections::HashMap;
use bp3d_sal::ast::RefResolver;
use bp3d_sal::ast::tree::{BlendfuncStatement, PipelineStatement, Property, Struct};

pub struct Ast<
    Pc = Property<usize>, Po = Property<usize>, Pb = Property<usize>,
    Sc = Struct<usize>, Sp = Struct<usize>, Sb = Struct<usize>, Sv = Struct<usize>
>
// where Pc is the property type for root constants
// Po the property type for outputs
// Pb the property type for objects
// Sc the struct type for root constants layout
// Sp the struct type for packed structs
// Sb the struct type for constant buffers
// Sv the struct type for vertex formats
{
    pub root_constants_layout: Option<Sc>,
    pub packed_structs: Vec<Sp>,
    //Root constants/push constants, emulated by global uniform buffer in GL targets
    pub root_constants: Vec<Pc>,
    //Fragment shader outputs/render target outputs
    pub outputs: Vec<Po>,
    //Samplers and textures
    pub objects: Vec<Pb>,
    pub cbuffers: Vec<Sb>,
    pub vformat: Option<Sv>,
    pub pipeline: Option<PipelineStatement>,
    pub blendfuncs: Vec<BlendfuncStatement>,
    packed_structs_by_name: HashMap<String, usize>,
    offset_packed_structs: usize
}

impl<Pc, Po, Pb, Sc, Sp, Sb, Sv> Ast<Pc, Po, Pb, Sc, Sp, Sb, Sv> {
    pub fn new() -> Ast<Pc, Po, Pb, Sc, Sp, Sb, Sv> {
        Ast {
            root_constants_layout: None,
            packed_structs: Vec::new(),
            root_constants: Vec::new(),
            outputs: Vec::new(),
            objects: Vec::new(),
            cbuffers: Vec::new(),
            vformat: None,
            pipeline: None,
            blendfuncs: Vec::new(),
            packed_structs_by_name: HashMap::new(),
            offset_packed_structs: 0
        }
    }

    pub fn remove_packed_struct(&mut self, index: usize) -> Sp {
        let obj = self.packed_structs.remove(index - self.offset_packed_structs);
        self.offset_packed_structs += 1;
        obj
    }

    /*pub fn map_root_constants_layout<E, Sc1, F: FnMut(Sc) -> Result<Sc1, E>>(self, f: F)
        -> Result<Ast<Pc, Po, Pb, Sc1, Sp, Sb, Sv>, E> {
        let root_constants_layout = match self.root_constants_layout {
            Some(v) => Some(f(v)?),
            None => None
        };
        Ok(Ast {
            root_constants_layout,
            ..self
        })
    }

    pub fn map_cbuffers<E, Sb1, F: FnMut(Sb) -> Result<Sb1, E>>(self, f: F)
        -> Result<Ast<Pc, Po, Pb, Sc, Sp, Sb1, Sv>, E> {
        let mut cbuffers = Vec::new();
        for v in self.cbuffers {
            cbuffers.push(f(v)?);
        }
        Ok(Ast {
            cbuffers,
            ..self
        })
    }

    pub fn map_packed_structs<E, Sp1, F: FnMut(Sp) -> Result<Sp1, E>>(self, f: F)
        -> Result<Ast<Pc, Po, Pb, Sc, Sp1, Sb, Sv>, E> {
        let mut packed_structs = Vec::new();
        for v in self.packed_structs {
            packed_structs.push(f(v)?);
        }
        Ok(Ast {
            packed_structs,
            ..self
        })
    }

    pub fn map_root_constants<E, Pc1, F: FnMut(Pc) -> Result<Pc1, E>>(self, f: F)
        -> Result<Ast<Pc1, Po, Pb, Sc, Sp, Sb, Sv>, E> {
        let mut root_constants = Vec::new();
        for v in self.root_constants {
            root_constants.push(f(v)?);
        }
        Ok(Ast {
            root_constants,
            ..self
        })
    }*/

    pub fn push_packed_struct(&mut self, name: String, st: Sp) -> usize {
        let id = self.packed_structs.len();
        self.packed_structs.push(st);
        self.packed_structs_by_name.insert(name, id);
        id
    }

    pub fn get_struct_ref(&self, id: usize) -> &Sp {
        &self.packed_structs[id - self.offset_packed_structs]
    }
}

impl<Pc, Po, Pb, Sc, Sp, Sb, Sv> RefResolver for Ast<Pc, Po, Pb, Sc, Sp, Sb, Sv> {
    type Key = usize;

    fn resolve_struct_ref(&self, name: &str) -> Option<Self::Key> {
        self.packed_structs_by_name.get(name).copied()
    }
}

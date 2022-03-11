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

use crate::ast::tree::{BlendfuncStatement, PipelineStatement, Property, Struct};

pub trait RefResolver {
    type Key;
    fn resolve_struct_ref(&self, name: &str) -> Option<Self::Key>;
}

pub trait Visitor<A: RefResolver> {
    type Error;
    fn visit_constant(&mut self, ast: &mut A, val: Property<A::Key>) -> Result<(), Self::Error>;
    fn visit_output(&mut self, ast: &mut A, val: Property<A::Key>) -> Result<(), Self::Error>;
    fn visit_constant_buffer(&mut self, ast: &mut A, val: Struct<A::Key>) -> Result<(), Self::Error>;
    fn visit_vertex_format(&mut self, ast: &mut A, val: Struct<A::Key>) -> Result<(), Self::Error>;
    fn visit_pipeline(&mut self, ast: &mut A, val: PipelineStatement) -> Result<(), Self::Error>;
    fn visit_blendfunc(&mut self, ast: &mut A, val: BlendfuncStatement) -> Result<(), Self::Error>;
    fn visit_noop(&mut self, ast: &mut A) -> Result<(), Self::Error>;
    fn visit_use(&mut self, ast: &mut A, module: String, member: String) -> Result<(), Self::Error>;
}

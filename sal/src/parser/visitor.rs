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

use std::fmt::Debug;
use crate::parser::tree::{Property, Struct, Use, VariableList};
use super::tree;

pub trait Visitor
{
    type Error: Debug;
    fn visit_constant(&mut self, val: tree::Property) -> Result<(), Self::Error>;
    fn visit_constant_buffer(&mut self, val: tree::Struct) -> Result<(), Self::Error>;
    fn visit_output(&mut self, val: tree::Property) -> Result<(), Self::Error>;
    fn visit_vertex_format(&mut self, val: tree::Struct) -> Result<(), Self::Error>;
    fn visit_use(&mut self, val: tree::Use) -> Result<(), Self::Error>;
    fn visit_pipeline(&mut self, val: tree::VariableList) -> Result<(), Self::Error>;
    fn visit_blendfunc(&mut self, val: tree::VariableList) -> Result<(), Self::Error>;
}

pub struct VecVisitor
{
    tree: Vec<tree::Root>
}

impl VecVisitor
{
    pub fn into_inner(self) -> Vec<tree::Root>
    {
        self.tree
    }

    pub fn new() -> VecVisitor
    {
        VecVisitor {
            tree: Vec::new()
        }
    }
}

impl Visitor for VecVisitor
{
    type Error = ();

    fn visit_constant(&mut self, val: Property) -> Result<(), Self::Error> {
        self.tree.push(tree::Root::Constant(val));
        Ok(())
    }

    fn visit_constant_buffer(&mut self, val: Struct) -> Result<(), Self::Error> {
        self.tree.push(tree::Root::ConstantBuffer(val));
        Ok(())
    }

    fn visit_output(&mut self, val: Property) -> Result<(), Self::Error> {
        self.tree.push(tree::Root::Output(val));
        Ok(())
    }

    fn visit_vertex_format(&mut self, val: Struct) -> Result<(), Self::Error> {
        self.tree.push(tree::Root::VertexFormat(val));
        Ok(())
    }

    fn visit_use(&mut self, val: Use) -> Result<(), Self::Error> {
        self.tree.push(tree::Root::Use(val));
        Ok(())
    }

    fn visit_pipeline(&mut self, val: VariableList) -> Result<(), Self::Error> {
        self.tree.push(tree::Root::Pipeline(val));
        Ok(())
    }

    fn visit_blendfunc(&mut self, val: VariableList) -> Result<(), Self::Error> {
        self.tree.push(tree::Root::Blendfunc(val));
        Ok(())
    }
}

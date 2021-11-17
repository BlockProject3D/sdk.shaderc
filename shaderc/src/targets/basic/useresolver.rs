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

use std::{collections::HashMap, path::Path};

use bpx::macros::impl_err_conversion;
use log::debug;
use sal::{
    ast::{tree::Statement, IgnoreUseResolver, UseResolver},
    utils::{auto_lexer_parser, AutoError}
};

use crate::targets::basic::shaderlib::ShaderLib;

#[derive(Debug)]
pub enum Error
{
    ShaderLib(crate::targets::basic::shaderlib::Error),
    Sal(AutoError<()>),
    ModuleNotFound(String),
    MemberNotFound(String)
}

impl_err_conversion!(
    Error {
        crate::targets::basic::shaderlib::Error => ShaderLib,
        AutoError<()> => Sal
    }
);

pub struct BasicUseResolver<'a>
{
    modules: HashMap<String, Vec<Statement>>,
    shader_libs: Vec<ShaderLib<'a>>
}

impl<'a> BasicUseResolver<'a>
{
    pub fn new(libs: &Vec<&'a Path>) -> Self
    {
        Self {
            modules: HashMap::new(),
            shader_libs: libs.into_iter().map(|l| ShaderLib::new(l)).collect()
        }
    }
}

impl<'a> UseResolver for BasicUseResolver<'a>
{
    type Error = Error;

    fn resolve(&mut self, item: sal::parser::tree::Use) -> Result<Statement, Self::Error>
    {
        if !self.modules.contains_key(&item.module) {
            let mut flag = false;
            for v in &mut self.shader_libs {
                if let Some(module) = v.try_load(&item.module)? {
                    let ast = auto_lexer_parser(module, IgnoreUseResolver {})?;
                    self.modules.insert(item.module.clone(), ast);
                    flag = true;
                    debug!("Successfully resolved module {}", item.module);
                }
            }
            if !flag {
                return Err(Error::ModuleNotFound(item.module));
            }
        }
        let ast = &self.modules[&item.module];
        let stmt = ast.iter().find(|e| e.get_name() == Some(&item.member));
        let stmt = stmt.ok_or_else(|| Error::MemberNotFound(item.member.clone()))?;
        debug!("Successfully resolved SAL use {}::{}", item.module, item.member);
        Ok(stmt.clone())
    }
}

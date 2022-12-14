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

use std::collections::{BTreeMap, HashMap, HashSet};
use bpx::shader::Stage;
use log::{debug, error, warn};
use bp3d_sal::ast::tree::{Attribute, PropertyType, Struct};
use crate::targets::basic::{BasicAst, ShaderToSal};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("multiple definitions of binding {0}")]
    RedefinedBinding(u32),
    #[error("multiple definitions of the same symbol")]
    RedefinedSymbol,
    #[error("unable to locate root constants layout")]
    NoRootConstants
}

pub struct ShaderStage
{
    pub statements: BasicAst,
    pub strings: Vec<rglslang::shader::Part>
}

#[derive(Debug)]
pub enum BindingType
{
    Texture,
    Sampler,
    CBuf
}

pub fn merge_stages(shaders: Vec<ShaderToSal>) -> BTreeMap<Stage, ShaderStage>
{
    let mut map = BTreeMap::new();
    for v in shaders {
        if !map.contains_key(&v.stage) {
            map.insert(v.stage, ShaderStage {
                statements: v.statements,
                strings: v.strings
            });
        } else {
            let stage = map.get_mut(&v.stage).unwrap();
            stage.strings.extend(v.strings);
            stage.statements.extend(v.statements);
        }
    }
    map
}

pub fn relocate_bindings<'a, F: FnMut(&'a str, BindingType, Option<u32>, u32) -> u32>(stages: &'a BTreeMap<Stage, ShaderStage>, mut func: F)
{
    let mut map = HashMap::new();
    stages.iter().for_each(|(_, v)| {
        for v in &v.statements.cbuffers {
            let mut cbuf_func = || {
                if let Some(attr) = &v.inner.attr {
                    if let Attribute::Order(slot) = attr {
                        v.external.set(true);
                        return func(&v.inner.name, BindingType::CBuf, Some(*slot), v.slot.get());
                    }
                }
                func(&v.inner.name, BindingType::CBuf, None, v.slot.get())
            };
            let fsk;
            if let Some(slot) = map.get(&v.inner.name) {
                fsk = *slot;
            } else {
                fsk = cbuf_func();
                map.insert(&v.inner.name, fsk);
            }
            debug!("CBuffer {} : {}", v.inner.name, fsk);
            v.slot.set(fsk);
        }
        for v in &v.statements.objects {
            let mut prop_func = |t: BindingType| {
                if let Some(attr) = &v.inner.pattr {
                    if let Attribute::Order(slot) = attr {
                        v.external.set(true);
                        return func(&v.inner.pname, t, Some(*slot), v.slot.get());
                    }
                }
                func(&v.inner.pname, t, None, v.slot.get())
            };
            let fsk;
            if let Some(slot) = map.get(&v.inner.pname) {
                fsk = *slot;
            } else {
                fsk = match v.inner.ptype {
                    PropertyType::Sampler => prop_func(BindingType::Sampler),
                    _ => prop_func(BindingType::Texture)
                };
                map.insert(&v.inner.pname, fsk);
            }
            debug!("Object {:?} {} : {}", v.inner.ptype, v.inner.pname, fsk);
            v.slot.set(fsk);
        }
    });
}

pub fn test_bindings<F: FnMut(BindingType, u32) -> bool>(stages: &BTreeMap<Stage, ShaderStage>, mut func: F) -> Result<(), Error>
{
    let mut map = HashMap::new();
    for v in stages.values() {
        if v.statements.root_constants_layout.is_some() && !func(BindingType::CBuf, 0) {
            error!("Redefinition of root constants layout");
            return Err(Error::RedefinedBinding(0))
        }
        for slot in &v.statements.cbuffers {
            if map.contains_key(&slot.inner.name) {
                continue;
            }
            if !func(BindingType::CBuf, slot.slot.get()) {
                error!("Constant buffer '{}' is attempting to relocate to {} which is already in use!", slot.inner.name, slot.slot.get());
                return Err(Error::RedefinedBinding(slot.slot.get()));
            }
            map.insert(&slot.inner.name, slot.slot.get());
        }
        for slot in &v.statements.objects {
            if map.contains_key(&slot.inner.pname) {
                continue;
            }
            if slot.inner.ptype != PropertyType::Sampler {
                if !func(BindingType::Sampler, slot.slot.get()) {
                    error!("Sampler '{}' is attempting to relocate to {} which is already in use!", slot.inner.pname, slot.slot.get());
                    return Err(Error::RedefinedBinding(slot.slot.get()));
                }
            } else {
                if !func(BindingType::Texture, slot.slot.get()) {
                    warn!("Texture '{}' is attempting to relocate to {} which is already in use!", slot.inner.pname, slot.slot.get());
                    return Err(Error::RedefinedBinding(slot.slot.get()));
                }
            }
            map.insert(&slot.inner.pname, slot.slot.get());
        }
    }
    Ok(())
}

pub fn test_symbols(stages: &BTreeMap<Stage, ShaderStage>) -> Result<(), Error>
{
    for (_, v) in stages {
        let mut set = HashSet::new();
        for v in &v.statements.cbuffers {
            if !set.insert(&v.inner.name) {
                error!("Multiple definitions of symbol '{}'", v.inner.name);
                return Err(Error::RedefinedSymbol);
            }
        }
        for v in &v.statements.objects {
            if !set.insert(&v.inner.pname) {
                error!("Multiple definitions of symbol '{}'", v.inner.pname);
                return Err(Error::RedefinedSymbol);
            }
        }
    }
    Ok(())
}

pub fn get_root_constants_layout(stages: &mut BTreeMap<Stage, ShaderStage>) -> Result<Struct<usize>, Error>
{
    let root_constants_layout = stages.iter_mut().find(|(_, v)| {
        if let Some(_) = &v.statements.root_constants_layout {
            true
        } else {
            false
        }
    }).ok_or_else(|| Error::NoRootConstants)?.1;
    Ok(root_constants_layout.statements.root_constants_layout.take().unwrap())
}

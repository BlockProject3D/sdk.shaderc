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
use log::warn;
use crate::targets::basic::{BindingType, relocate_bindings, ShaderStage, test_bindings};

//TODO: In VK target ensure that all bindings are unique across all types of bindings
pub fn gl_relocate_bindings(stages: &mut BTreeMap<Stage, ShaderStage>)
{
    let mut cbufs = HashSet::new();
    let mut textures = HashSet::new();
    let mut samplers = HashSet::new();
    let mut cbufs_name = HashMap::new();
    let mut samplers_name = HashMap::new();
    let mut textures_name = HashMap::new();
    let mut cbuf_counter: u32 = 1;
    let mut sampler_counter: u32 = 0;
    let mut texture_counter: u32 = 0;
    let mut insert_texture = |name, slot| {
        if !textures.insert(slot) {
            warn!("Possible duplicate of texture slot {}", slot);
        }
        textures_name.insert(slot, name);
    };
    let mut insert_sampler = |name, slot| {
        if !samplers.insert(slot) {
            warn!("Possible duplicate of sampler slot {}", slot);
        }
        samplers_name.insert(slot, name);
    };
    let mut insert_cbuffer = |name, slot| {
        if !cbufs.insert(slot) {
            warn!("Possible duplicate of constant buffer slot {}", slot);
        }
        cbufs_name.insert(slot, name);
    };
    relocate_bindings(stages, |name, t, existing, _| {
        match t {
            BindingType::Texture => {
                let slot = existing.map(|slot| {
                    texture_counter = slot + 1;
                    slot
                }).unwrap_or_else(|| {
                    texture_counter += 1;
                    texture_counter - 1
                });
                insert_texture(name, slot);
                slot
            },
            BindingType::Sampler => {
                let slot = existing.map(|slot| {
                    sampler_counter = slot + 1;
                    slot
                }).unwrap_or_else(|| {
                    sampler_counter += 1;
                    sampler_counter - 1
                });
                insert_sampler(name, slot);
                slot
            },
            BindingType::CBuf => {
                let slot = existing.map(|slot| {
                    cbuf_counter = slot + 1;
                    slot
                }).unwrap_or_else(|| {
                    cbuf_counter += 1;
                    cbuf_counter - 1
                });
                insert_cbuffer(name, slot);
                slot
            }
        }
    });
    relocate_bindings(stages, |name, t, existing, mut current| {
        match t {
            BindingType::Texture => {
                if let Some(slot) = existing {
                    slot
                } else {
                    if let Some(name1) = textures_name.get(&current) {
                        if name1 == &name {
                            return current
                        }
                    }
                    while textures.contains(&current) {
                        current += 1;
                    }
                    current
                }
            },
            BindingType::Sampler => {
                if let Some(slot) = existing {
                    slot
                } else {
                    if let Some(name1) = samplers_name.get(&current) {
                        if name1 == &name {
                            return current
                        }
                    }
                    while samplers.contains(&current) {
                        current += 1;
                    }
                    current
                }
            },
            BindingType::CBuf => {
                if let Some(slot) = existing {
                    slot
                } else {
                    if let Some(name1) = cbufs_name.get(&current) {
                        if name1 == &name {
                            return current
                        }
                    }
                    while cbufs.contains(&current) {
                        current += 1;
                    }
                    current
                }
            }
        }
    });
}

pub fn gl_test_bindings(stages: &BTreeMap<Stage, ShaderStage>) -> Result<(), crate::targets::basic::sal_compiler::Error>
{
    let mut cbufs = HashSet::new();
    let mut textures = HashSet::new();
    let mut samplers = HashSet::new();
    test_bindings(stages, |t, slot| {
        match t {
            BindingType::Texture => textures.insert(slot),
            BindingType::Sampler => samplers.insert(slot),
            BindingType::CBuf => cbufs.insert(slot),
        }
    })
}

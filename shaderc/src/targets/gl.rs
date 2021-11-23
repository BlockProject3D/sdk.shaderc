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
use bp3d_threads::{ScopedThreadManager, ThreadPool};
use bpx::shader::Stage;
use log::{debug, error, info, trace, warn};
use rglslang::environment::{Client, Environment};
use rglslang::shader::{Messages, Profile};
use crate::options::{Args, Error};
use crate::targets::basic::{BindingType, get_root_constants_layout, relocate_bindings, ShaderStage, test_bindings};
use crate::targets::sal_to_glsl::translate_sal_to_glsl;

pub struct EnvInfo
{
    pub gl_version_str: &'static str,
    pub gl_version_int: i32,
    pub explicit_bindings: bool
}

pub fn compile_stages(env: &EnvInfo, args: &Args, mut stages: BTreeMap<Stage, ShaderStage>) -> Result<(), Error>
{
    let root_constants_layout = &get_root_constants_layout(&mut stages)?;
    let root = crossbeam::scope(|scope| {
        let mut root = Vec::new();
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<(), Error>> = ThreadPool::new(args.n_threads);
        info!("Initialized thread pool with {} max thread(s)", args.n_threads);
        for (stage, mut shader) in stages {
            pool.dispatch(&manager, move |_| {
                debug!("Translating SAL AST for stage {:?} to GLSL for OpenGL {}...", stage, env.gl_version_str);
                let glsl = translate_sal_to_glsl(env.explicit_bindings, &root_constants_layout, &shader.statements)?;
                info!("Translated GLSL: \n{}", glsl);
                shader.strings.insert(0, rglslang::shader::Part::new_with_name(glsl, "__internal_sal__"));
                shader.strings.insert(0, rglslang::shader::Part::new_with_name(format!("#version {} core\n", env.gl_version_int), "__internal_glsl_version__"));
                let strings = shader.strings.clone();
                trace!("Shader strings: \n{:?}", strings);
                let rst = match stage {
                    Stage::Vertex => rglslang::environment::Stage::Vertex,
                    Stage::Hull => rglslang::environment::Stage::Hull,
                    Stage::Domain => rglslang::environment::Stage::Domain,
                    Stage::Geometry => rglslang::environment::Stage::Geometry,
                    Stage::Pixel => rglslang::environment::Stage::Pixel
                };
                let msgs;
                if args.debug {
                    msgs = Messages::new().debug().ast();
                } else {
                    msgs = Messages::new();
                }
                let mut builder = rglslang::shader::Builder::new(Environment::new_opengl(rst, Client::OpenGL, Some(env.gl_version_int)))
                    .messages(msgs)
                    .entry_point("main")
                    .source_entry_point("main")
                    .default_version(env.gl_version_int)
                    .default_profile(Profile::Core);
                for v in strings {
                    builder = builder.add_part(v);
                }
                let rshader = builder.parse();
                if !rshader.check() {
                    error!("GLSL has reported the following error: \n{}", rshader.get_info_log());
                    return Err(Error::new("error parsing GLSL"));
                } else {
                    info!("Successfully parsed GLSL code");
                    info!("Shader log: \n{}", rshader.get_info_log());
                    info!("Shader debug log: \n{}", rshader.get_info_debug_log());
                }
                Ok(())
            });
            debug!("Dispatch stage {:?}", stage);
        }
        pool.join().unwrap();
        while let Some(res) = pool.poll() {
            root.push(res);
        }
        root
    }).unwrap();
    for v in root {
        v?;
    }
    Ok(())
}

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

pub fn gl_test_bindings(stages: &BTreeMap<Stage, ShaderStage>) -> Result<(), Error>
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

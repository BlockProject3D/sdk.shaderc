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
use rglslang::shader::{Messages, Profile, Shader};
use sal::ast::tree::{BlendfuncStatement, PipelineStatement, Property};
use crate::options::{Args, Error};
use crate::targets::basic::{BindingType, get_root_constants_layout, relocate_bindings, ShaderStage, Slot, test_bindings};
use crate::targets::layout140::{compile_packed_structs, compile_struct, StructOffset};
use crate::targets::sal_to_glsl::translate_sal_to_glsl;

const MAX_CBUFFER_SIZE: usize = 65536;
const MAX_ROOT_CONSTANTS_SIZE: usize = 128;

pub struct EnvInfo
{
    pub gl_version_str: &'static str,
    pub gl_version_int: i32,
    pub explicit_bindings: bool
}

pub struct Symbols
{
    pub root_constant_layout: StructOffset,
    pub packed_structs: Vec<StructOffset>,
    pub cbuffers: Vec<Slot<StructOffset>>,
    pub outputs: Vec<Slot<Property>>, //Fragment shader outputs/render target outputs
    pub objects: Vec<Slot<Property>>, //Samplers and textures
    pub pipeline: Option<PipelineStatement>,
    pub blendfuncs: Vec<BlendfuncStatement>
}

pub struct ShaderData
{
    strings: Vec<rglslang::shader::Part>,
    shader: Shader,
    stage: Stage
}

pub struct ShaderData1
{
    pub strings: Vec<rglslang::shader::Part>,
    pub stage: Stage
}

pub struct CompiledShaderStage
{
    pub packed_structs: HashMap<String, StructOffset>,
    pub cbuffers: Vec<Slot<StructOffset>>,
    pub outputs: Vec<Slot<Property>>, //Fragment shader outputs/render target outputs
    pub objects: Vec<Slot<Property>>, //Samplers and textures
    pub pipeline: Option<PipelineStatement>,
    pub blendfuncs: Vec<BlendfuncStatement>,
    pub strings: Vec<rglslang::shader::Part>,
    pub shader: Shader,
    pub stage: Stage
}

pub struct CompileOutput
{
    pub root_constant_layout: StructOffset,
    pub stages: Vec<CompiledShaderStage>
}

fn build_messages(args: &Args) -> Messages
{
    let msgs;
    if args.debug {
        msgs = Messages::new().debug().ast();
    } else {
        msgs = Messages::new();
    }
    msgs
}

pub fn compile_stages(env: &EnvInfo, args: &Args, mut stages: BTreeMap<Stage, ShaderStage>) -> Result<CompileOutput, Error>
{
    let root_constants_layout = get_root_constants_layout(&mut stages)?;
    let root = crossbeam::scope(|scope| {
        let mut root = Vec::new();
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<CompiledShaderStage, Error>> = ThreadPool::new(args.n_threads);
        info!("Initialized thread pool with {} max thread(s)", args.n_threads);
        let root_constants_layout = &root_constants_layout;
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
                let msgs = build_messages(args);
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
                }
                info!("Successfully parsed GLSL code");
                info!("Shader log: \n{}", rshader.get_info_log());
                info!("Shader debug log: \n{}", rshader.get_info_debug_log());
                let packed_structs = compile_packed_structs(shader.statements.packed_structs)?;
                let mut cbuffers = Vec::new();
                for v in shader.statements.cbuffers {
                    let inner = compile_struct(v.inner, &packed_structs)?;
                    debug!("Size of constant buffer '{}' is {} bytes", inner.name, inner.size);
                    if inner.size > MAX_CBUFFER_SIZE { // Check if UBO exceeds maximum size
                        error!("The size of a constant buffer cannot exceed 65536 bytes after alignment, however constant buffer '{}' takes {} bytes after alignment", inner.name, inner.size);
                        return Err(Error::new("constant buffer size overload"));
                    }
                    cbuffers.push(Slot {
                        inner,
                        slot: v.slot,
                        explicit: v.explicit
                    });
                }
                let compiled = CompiledShaderStage {
                    cbuffers,
                    packed_structs,
                    outputs: shader.statements.outputs,
                    objects: shader.statements.objects,
                    pipeline: shader.statements.pipeline,
                    blendfuncs: shader.statements.blendfuncs,
                    strings: shader.strings,
                    shader: rshader,
                    stage
                };
                Ok(compiled)
            });
            debug!("Dispatch stage {:?}", stage);
        }
        pool.join().unwrap();
        while let Some(res) = pool.poll() {
            root.push(res);
        }
        root
    }).unwrap();
    let dummy = HashMap::new();
    let compiled_root_constants = compile_struct(root_constants_layout, &dummy)?;
    debug!("Size of root constants layout is {} bytes", compiled_root_constants.size);
    if compiled_root_constants.size > MAX_ROOT_CONSTANTS_SIZE {
        warn!("Root constants layout size ({} bytes) exceeds the recommended limit of 128 bytes after alignment", compiled_root_constants.size);
    }
    let mut stages = Vec::new();
    for v in root {
        let stage = v?;
        stages.push(stage);
    }
    Ok(CompileOutput {
        stages,
        root_constant_layout: compiled_root_constants
    })
}

fn merge_symbols(output: CompileOutput) -> (Symbols, Vec<ShaderData>)
{
    let mut symbols = HashMap::new();
    let mut check_insert_symbol = |name: &String, slot| {
        let mut flag = false;
        if let Some(s) = symbols.get(name) {
            if *s != slot {
                warn!("Duplicate symbol name '{}'", name);
            }
            flag = true;
        }
        symbols.insert(name.clone(), slot);
        flag
    };
    let mut shaders = Vec::new();
    let mut cbuffers = Vec::new();
    let mut outputs = Vec::new();
    let mut objects = Vec::new();
    let mut pipeline = None;
    let mut blendfuncs = Vec::new();
    let mut packed_structs = Vec::new();
    for v in output.stages {
        for v in v.objects {
            if !check_insert_symbol(&v.inner.pname, v.slot.get()) {
                objects.push(v);
            }
        }
        for v in v.outputs {
            if !check_insert_symbol(&v.inner.pname, v.slot.get()) {
                outputs.push(v);
            }
        }
        for v in v.cbuffers {
            if !check_insert_symbol(&v.inner.name, v.slot.get()) {
                cbuffers.push(v);
            }
        }
        for (i, v) in v.blendfuncs.into_iter().enumerate() {
            if !check_insert_symbol(&v.name, i as u32) {
                blendfuncs.push(v);
            }
        }
        if let Some(p) = v.pipeline {
            if pipeline.is_some() {
                warn!("Duplicate symbol name '{}'", p.name)
            } else {
                pipeline = Some(p);
            }
        }
        for (i, (_, v)) in v.packed_structs.into_iter().enumerate() {
            if !check_insert_symbol(&v.name, i as u32) {
                packed_structs.push(v);
            }
        }
        shaders.push(ShaderData {
            shader: v.shader,
            stage: v.stage,
            strings: v.strings
        });
    }
    let syms = Symbols {
        cbuffers,
        packed_structs,
        outputs,
        objects,
        pipeline,
        blendfuncs,
        root_constant_layout: output.root_constant_layout
    };
    (syms, shaders)
}

pub fn link_shaders(args: &Args, output: CompileOutput) -> Result<(Symbols, Vec<ShaderData1>), Error>
{
    let (syms, shaders) = merge_symbols(output);
    let mut shaders1 = Vec::with_capacity(shaders.len());
    let msgs = build_messages(args);
    let mut builder = rglslang::program::Builder::new()
        .messages(msgs);
    for v in shaders {
        shaders1.push(ShaderData1 {
            strings: v.strings,
            stage: v.stage
        });
        builder = builder.add_shader(v.shader);
    }
    let prog = builder.link();
    if !prog.check() {
        error!("GLSL has reported the following error: \n{}", prog.get_info_log());
        return Err(Error::new("error linking GLSL"));
    }
    info!("Successfully linked GLSL shaders");
    info!("Shader log: \n{}", prog.get_info_log());
    info!("Shader debug log: \n{}", prog.get_info_debug_log());
    Ok((syms, shaders1))
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

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

use std::collections::{BTreeMap, HashMap};
use bp3d_threads::{ScopedThreadManager, ThreadPool};
use bpx::shader::Stage;
use log::{debug, error, info, trace, warn};
use rglslang::environment::{Client, Environment};
use rglslang::shader::{Messages, Profile, Shader};
use bp3d_sal::ast::tree::{BlendfuncStatement, PipelineStatement, Property, Struct};
use crate::config::Config;
use crate::options::{Error};
use crate::targets::basic::{get_root_constants_layout, ShaderStage, Slot};
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

pub struct Object<T>
{
    pub inner: Slot<T>,
    pub stage_vertex: bool,
    pub stage_hull: bool,
    pub stage_domain: bool,
    pub stage_geometry: bool,
    pub stage_pixel: bool
}

impl<T> Object<T>
{
    pub fn new(inner: Slot<T>) -> Object<T>
    {
        Object {
            inner,
            stage_vertex: false,
            stage_hull: false,
            stage_domain: false,
            stage_geometry: false,
            stage_pixel: false
        }
    }

    pub fn mark_stage(&mut self, s: Stage)
    {
        match s {
            Stage::Vertex => self.stage_vertex = true,
            Stage::Hull => self.stage_hull = true,
            Stage::Domain => self.stage_domain = true,
            Stage::Geometry => self.stage_geometry = true,
            Stage::Pixel => self.stage_pixel = true
        }
    }
}

pub struct Symbols
{
    pub root_constant_layout: StructOffset,
    pub packed_structs: Vec<StructOffset>,
    pub cbuffers: Vec<Object<StructOffset>>,
    pub outputs: Vec<Slot<Property<usize>>>, //Fragment shader outputs/render target outputs
    pub objects: Vec<Object<Property<usize>>>, //Samplers and textures
    pub pipeline: Option<PipelineStatement>,
    pub vformat: Option<Struct<usize>>,
    pub blendfuncs: Vec<BlendfuncStatement>
}

pub struct ShaderData
{
    strings: Vec<rglslang::shader::Part>,
    shader: Shader,
    stage: Stage
}

pub struct ShaderBytes
{
    pub data: Vec<u8>,
    pub stage: Stage
}

pub struct CompiledShaderStage
{
    pub packed_structs: Vec<StructOffset>,
    pub cbuffers: Vec<Slot<StructOffset>>,
    pub outputs: Vec<Slot<Property<usize>>>, //Fragment shader outputs/render target outputs
    pub objects: Vec<Slot<Property<usize>>>, //Samplers and textures
    pub pipeline: Option<PipelineStatement>,
    pub vformat: Option<Struct<usize>>,
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

fn build_messages(config: &Config) -> Messages
{
    let msgs;
    if config.debug {
        msgs = Messages::new().debug().ast();
    } else {
        msgs = Messages::new();
    }
    msgs
}

pub fn compile_stages(env: &EnvInfo, config: &Config, mut stages: BTreeMap<Stage, ShaderStage>) -> Result<CompileOutput, Error>
{
    let root_constants_layout = get_root_constants_layout(&mut stages)?;
    let stages: Result<Vec<CompiledShaderStage>, Error> = crossbeam::scope(|scope| {
        let manager = ScopedThreadManager::new(scope);
        let mut pool: ThreadPool<ScopedThreadManager, Result<CompiledShaderStage, Error>> = ThreadPool::new(config.n_threads);
        info!("Initialized thread pool with {} max thread(s)", config.n_threads);
        let root_constants_layout = &root_constants_layout;
        for (stage, mut shader) in stages {
            pool.send(&manager, move |_| {
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
                let msgs = build_messages(config);
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
                        external: v.external
                    });
                }
                let compiled = CompiledShaderStage {
                    cbuffers,
                    packed_structs,
                    outputs: shader.statements.outputs,
                    objects: shader.statements.objects,
                    pipeline: shader.statements.pipeline,
                    blendfuncs: shader.statements.blendfuncs,
                    vformat: shader.statements.vformat,
                    strings: shader.strings,
                    shader: rshader,
                    stage
                };
                Ok(compiled)
            });
            debug!("Dispatch stage {:?}", stage);
        }
        pool.reduce().map(|v| v.unwrap()).collect()
    }).unwrap();
    let dummy = Vec::new();
    let compiled_root_constants = compile_struct(root_constants_layout, &dummy)?;
    debug!("Size of root constants layout is {} bytes", compiled_root_constants.size);
    if compiled_root_constants.size > MAX_ROOT_CONSTANTS_SIZE {
        warn!("Root constants layout size ({} bytes) exceeds the recommended limit of 128 bytes after alignment", compiled_root_constants.size);
    }
    Ok(CompileOutput {
        stages: stages?,
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
    let mut cbuffers = HashMap::new(); // Well rust wants to be slow
    // If rust lifetime system wasn't broken &str or &String would have worked!
    let mut outputs = Vec::new();
    let mut objects = HashMap::new(); // Well rust wants to be slow
    // If rust lifetime system wasn't broken &str or &String would have worked!
    let mut pipeline = None;
    let mut vformat = None;
    let mut blendfuncs = Vec::new();
    let mut packed_structs = Vec::new();
    for stage in output.stages {
        for v in stage.objects {
            let obj = objects.entry(v.inner.pname.clone()).or_insert_with(|| Object::new(v));
            obj.mark_stage(stage.stage);
        }
        for v in stage.outputs {
            if !check_insert_symbol(&v.inner.pname, v.slot.get()) {
                outputs.push(v);
            }
        }
        for v in stage.cbuffers {
            let obj = cbuffers.entry(v.inner.name.clone()).or_insert_with(|| Object::new(v));
            obj.mark_stage(stage.stage);
        }
        for (i, v) in stage.blendfuncs.into_iter().enumerate() {
            if !check_insert_symbol(&v.name, i as u32) {
                blendfuncs.push(v);
            }
        }
        if let Some(p) = stage.pipeline {
            if pipeline.is_some() {
                warn!("Ignoring duplicate pipeline with name '{}'", p.name)
            } else {
                pipeline = Some(p);
            }
        }
        if let Some(v) = stage.vformat {
            if vformat.is_some() {
                warn!("Ignoring duplicate vertex format with name '{}'", v.name)
            } else {
                vformat = Some(v);
            }
        }
        for (i, v) in stage.packed_structs.into_iter().enumerate() {
            if !check_insert_symbol(&v.name, i as u32) {
                packed_structs.push(v);
            }
        }
        shaders.push(ShaderData {
            shader: stage.shader,
            stage: stage.stage,
            strings: stage.strings
        });
    }
    let syms = Symbols {
        cbuffers: cbuffers.into_iter().map(|(_, v)| v).collect(),
        packed_structs,
        outputs,
        objects: objects.into_iter().map(|(_, v)| v).collect(),
        pipeline,
        vformat,
        blendfuncs,
        root_constant_layout: output.root_constant_layout
    };
    (syms, shaders)
}

/// This function links shaders only for pure OpenGL targets; vulkan and SpvCross based targets
/// aren't supported by this function.
pub fn gl_link_shaders(config: &Config, output: CompileOutput) -> Result<(Symbols, Vec<ShaderBytes>), Error>
{
    let (syms, shaders) = merge_symbols(output);
    let mut shaders1 = Vec::with_capacity(shaders.len());
    let msgs = build_messages(config);
    let mut builder = rglslang::program::Builder::new()
        .messages(msgs);
    for v in shaders {
        let data = v.strings.into_iter().map(|v| v.into_code()).collect::<Vec<_>>().join("");
        shaders1.push(ShaderBytes {
            data: data.into_bytes(),
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

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

use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use bp3d_sal::ast::tree::{BlendfuncStatement, PipelineStatement, Property, PropertyType, Struct};
use bp3d_symbols::{BlendfuncObject, ConstantObject, ConstPropType, OutputObject, OutputPropType};
use crate::targets::gl::core::{Object, ShaderBytes, Symbols};
use bpx::shader;
use bpx::shader::{ShaderPack, Stage, Type};
use log::{debug, error, info, warn};
use crate::targets::basic::Slot;
use crate::targets::gl::ext_data::{SymbolWriter, ToObject};
use crate::targets::layout140::StructOffset;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported binding register number")]
    UnsupportedBinding,
    #[error("illegal object type")]
    IllegalObject,
    #[error("illegal render target output type")]
    IllegalOutput,
    #[error("illegal constant type")]
    IllegalConstant,
    #[error("bpx error: {0}")]
    Bpx(bpx::shader::error::Error),
    #[error("bpx serde error: {0}")]
    Serde(bpx::sd::serde::Error)
}

fn build_blendfunc_lookup_map(blendfuncs: Vec<BlendfuncStatement>) -> HashMap<String, BlendfuncObject>
{
    let mut map = HashMap::new();
    for fnc in blendfuncs {
        map.insert(fnc.name, BlendfuncObject {
            alpha_op: fnc.alpha_op,
            src_color: fnc.src_color,
            dst_color: fnc.dst_color,
            src_alpha: fnc.src_alpha,
            dst_alpha: fnc.dst_alpha,
            color_op: fnc.color_op
        });
    }
    map
}

pub struct BpxWriter
{
    debug: bool,
    bpx: Option<ShaderPack<BufWriter<File>>>
}

impl BpxWriter {
    pub fn new(file: File, target: shader::Target, debug: bool) -> BpxWriter {
        let bpx = ShaderPack::create(BufWriter::new(file), shader::Builder::new()
            .ty(Type::Pipeline)
            .target(target));
        BpxWriter {
            debug,
            bpx: Some(bpx)
        }
    }

    fn write_objects(&self, bpx: &mut SymbolWriter<BufWriter<File>>, objects: Vec<Object<Property<usize>>>) -> Result<(), Error>
    {
        for sym in objects {
            let mut builder = shader::symbol::Builder::new(sym.inner.inner.pname);
            let slot = sym.inner.slot.get();
            if slot > 32 {
                error!("OpenGL limits texture/sampler bindings to 32, got a binding at register {}", slot);
                return Err(Error::UnsupportedBinding);
            } else if slot > 16 {
                warn!("This shader needs more than 16 bindings, this may not work on all hardware");
            }
            builder.register(slot as _);
            match sym.inner.inner.ptype {
                PropertyType::Sampler => builder.ty(shader::symbol::Type::Sampler),
                PropertyType::Texture2D(_) | PropertyType::Texture3D(_) | PropertyType::Texture2DArray(_)
                | PropertyType::TextureCube(_) => builder.ty(shader::symbol::Type::Texture),
                p => {
                    error!("Unsupported object type: {}", p);
                    return Err(Error::IllegalObject);
                }
            };
            builder.extended_data(sym.inner.inner.ptype.to_bpx_object(self.debug, &()).map_err(Error::Serde)?);
            if sym.inner.external.get() {
                builder.external(); //Global binding (goes in the global descriptor set)
            } else {
                builder.internal(); //Local binding (goes in the local descriptor set)
            }
            crate::targets::gl::ext_data::append_stages!(sym > builder);
            bpx.write(builder).map_err(Error::Bpx)?;
        }
        Ok(())
    }

    fn write_packed_structs(&self, bpx: &mut SymbolWriter<BufWriter<File>>, structs: &Vec<Slot<StructOffset>>) -> Result<(), Error>
    {
        for sym in structs {
            let mut builder = shader::symbol::Builder::new(sym.inner.name.clone());
            builder
                .ty(shader::symbol::Type::ConstantBuffer)
                .extended_data(sym.inner.to_bpx_object(self.debug, &(bpx, structs)).map_err(Error::Serde)?);
            if sym.external.get() {
                builder.external();
            } else {
                builder.internal();
            }
            bpx.write(builder).map_err(Error::Bpx)?;
        }
        Ok(())
    }

    fn write_cbuffers(&self, bpx: &mut SymbolWriter<BufWriter<File>>, objects: Vec<Object<StructOffset>>, packed_structs: &Vec<Slot<StructOffset>>) -> Result<(), Error>
    {
        for sym in objects {
            //Unfortunately we must clone because rust is unable to see that sym.inner.inner.name is
            // not used by to_bpx_object...
            let mut builder = shader::symbol::Builder::new(sym.inner.inner.name.clone());
            let slot = sym.inner.slot.get();
            if slot > 32 {
                error!("OpenGL limits texture/sampler bindings to 32, got a binding at register {}", slot);
                return Err(Error::UnsupportedBinding);
            } else if slot > 16 {
                warn!("This shader needs more than 16 bindings, this may not work on all hardware");
            }
            builder
                .register(slot as _)
                .ty(shader::symbol::Type::ConstantBuffer)
                .extended_data(sym.inner.inner.to_bpx_object(self.debug, &(bpx, packed_structs)).map_err(Error::Serde)?);
            if sym.inner.external.get() {
                builder.external();
            } else {
                builder.internal();
            }
            crate::targets::gl::ext_data::append_stages!(sym > builder);
            bpx.write(builder).map_err(Error::Bpx)?;
        }
        Ok(())
    }

    fn write_vformat(&self, bpx: &mut SymbolWriter<BufWriter<File>>, vformat: Option<Struct<usize>>) -> Result<(), Error>
    {
        if let Some(sym) = vformat {
            //Unfortunately we must clone because rust is unable to see that sym.name is
            // not used by to_bpx_object...
            let mut builder = shader::symbol::Builder::new(sym.name.clone());
            builder
                .external()
                .ty(shader::symbol::Type::VertexFormat)
                .extended_data(sym.to_bpx_object(self.debug, &()).map_err(Error::Serde)?);
            bpx.write(builder).map_err(Error::Bpx)?;
        } else {
            warn!("No vertex format was found in shader pack build");
        }
        Ok(())
    }

    fn write_pipeline(&self, bpx: &mut SymbolWriter<BufWriter<File>>, pipeline: Option<PipelineStatement>) -> Result<(), Error>
    {
        if let Some(sym) = pipeline {
            //Unfortunately we must clone because rust is unable to see that sym.name is
            // not used by to_bpx_object...
            let mut builder = shader::symbol::Builder::new(sym.name.clone());
            builder
                .internal()
                .ty(shader::symbol::Type::Pipeline)
                .extended_data(sym.to_bpx_object(self.debug, &()).map_err(Error::Serde)?);
            bpx.write(builder).map_err(Error::Bpx)?;
        } else {
            warn!("No pipeline was found in shader pack build");
        }
        Ok(())
    }

    fn write_outputs(&self, bpx: &mut SymbolWriter<BufWriter<File>>, outputs: Vec<Slot<Property<usize>>>, blendfuncs: Vec<BlendfuncStatement>) -> Result<(), Error>
    {
        if outputs.len() <= 0 {
            warn!("No render target outputs was found in shader pack build");
            return Ok(());
        }
        let funcs = build_blendfunc_lookup_map(blendfuncs);
        for sym in outputs {
            let output = OutputObject {
                blendfunc: funcs.get(&sym.inner.pname).map(|v| v.clone()),
                ty: match sym.inner.ptype {
                    PropertyType::Scalar(v) => OutputPropType::Scalar(v),
                    PropertyType::Vector(v) => OutputPropType::Vector(v),
                    s => {
                        error!("Requested type '{}' for a render target which isn't supported in OpenGL", s);
                        return Err(Error::IllegalOutput);
                    }
                }
            };
            let mut builder = shader::symbol::Builder::new(sym.inner.pname);
            builder
                .internal()
                .ty(shader::symbol::Type::Output)
                .register(sym.slot.get() as _)
                .extended_data(output.to_bpx_object(self.debug, &()).map_err(Error::Serde)?);
            bpx.write(builder).map_err(Error::Bpx)?;
        }
        Ok(())
    }

    fn write_root_constants(&self, bpx: &mut SymbolWriter<BufWriter<File>>, root_constants_layout: StructOffset) -> Result<(), Error>
    {
        for sym in root_constants_layout.props {
            let mut builder = shader::symbol::Builder::new(sym.inner.pname);
            builder.ty(shader::symbol::Type::Constant).external();
            let obj = ConstantObject {
                size: sym.size as _,
                offset: sym.aligned_offset as _,
                ty: match sym.inner.ptype {
                    PropertyType::Scalar(v) => ConstPropType::Scalar(v),
                    PropertyType::Vector(v) => ConstPropType::Vector(v),
                    PropertyType::Matrix(v) => ConstPropType::Matrix(v),
                    s => {
                        error!("Requested type '{}' for a constant which isn't supported in OpenGL", s);
                        return Err(Error::IllegalConstant);
                    }
                }
            };
            builder.extended_data(obj.to_bpx_object(self.debug, &()).map_err(Error::Serde)?);
            bpx.write(builder).map_err(Error::Bpx)?;
        }
        Ok(())
    }

    fn propagate_external_flag(&self, cbuffers: &Vec<Object<StructOffset>>, packed_structs: &Vec<Slot<StructOffset>>) {
        info!("Applying external flags on symbol references...");
        for cbuffer in cbuffers {
            for p in &cbuffer.inner.inner.props {
                match p.ptype {
                    PropertyType::StructRef(v) => {
                        let st = &packed_structs[v];
                        if cbuffer.inner.external.get() {
                            st.external.set(true);
                            debug!("Set external flag for '{}'", st.inner.name);
                        }
                    },
                    PropertyType::Array(v) => {
                        match v.item {
                            bp3d_sal::ast::tree::ArrayItemType::StructRef(v) => {
                                let st = &packed_structs[v];
                                if cbuffer.inner.external.get() {
                                    st.external.set(true);
                                    debug!("Set external flag for '{}'", st.inner.name);
                                }
                            },
                            _ => ()
                        }
                    },
                    _ => ()
                };
            }
        }
    }

    pub fn write_symbols(&mut self, syms: Symbols) -> Result<(), Error> {
        //The unwrap should be fine because bpx is initialized in new.
        // This unwrap may panic if write_symbols panics before putting bpx back.
        let packed_structs = syms.packed_structs.into_iter().map(Slot::new).collect();
        self.propagate_external_flag(&syms.cbuffers, &packed_structs);
        let mut writer = SymbolWriter::new(self.bpx.take().unwrap());
        self.write_objects(&mut writer, syms.objects)?;
        self.write_packed_structs(&mut writer, &packed_structs)?;
        self.write_cbuffers(&mut writer, syms.cbuffers, &packed_structs)?;
        self.write_vformat(&mut writer, syms.vformat)?;
        self.write_pipeline(&mut writer, syms.pipeline)?;
        self.write_outputs(&mut writer, syms.outputs, syms.blendfuncs)?;
        self.write_root_constants(&mut writer, syms.root_constant_layout)?;
        self.bpx = Some(writer.into_inner());
        Ok(())
    }

    pub fn write_shaders(&mut self, shaders: Vec<ShaderBytes>) -> Result<(), Error> {
        //The unwrap should be fine because bpx is initialized in new.
        // This unwrap may panic if write_symbols panics before putting bpx back.
        let mut tbl = self.bpx.as_mut().unwrap().shaders_mut();
        for stage in shaders {
            tbl.create(shader::Shader {
                stage: stage.stage,
                data: stage.data
            }).map_err(Error::Bpx)?;
        }
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), Error> {
        //The unwrap should be fine because bpx is initialized in new.
        // This unwrap may panic if write_symbols panics before putting bpx back.
        self.bpx.as_mut().unwrap().save().map_err(Error::Bpx)?;
        Ok(())
    }
}

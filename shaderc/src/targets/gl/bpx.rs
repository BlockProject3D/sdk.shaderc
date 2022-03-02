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
use crate::options::Error;
use bp3d_symbols::{BlendfuncObject, ConstantObject, ConstPropType, OutputObject, OutputPropType};
use crate::targets::gl::core::{Object, ShaderBytes, Symbols};
use bpx::shader;
use bpx::shader::{ShaderPack, Stage};
use log::{error, warn};
use crate::targets::basic::Slot;
use crate::targets::gl::ext_data::{SymbolWriter, ToObject};
use crate::targets::layout140::StructOffset;

fn write_objects(bpx: &mut SymbolWriter<BufWriter<File>>, objects: Vec<Object<Property>>, debug: bool) -> Result<(), Error>
{
    for sym in objects {
        let mut builder = shader::symbol::Builder::new(sym.inner.inner.pname);
        let slot = sym.inner.slot.get();
        if slot > 32 {
            error!("OpenGL limits texture/sampler bindings to 32, got a binding at register {}", slot);
            return Err(Error::new("unsupported binding register number"));
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
                return Err(Error::new("unsupported object type"));
            }
        };
        builder.extended_data(sym.inner.inner.ptype.to_bpx_object(debug, &())?);
        if sym.inner.explicit.get() {
            builder.external(); //Global binding (goes in the global descriptor set)
        } else {
            builder.internal(); //Local binding (goes in the local descriptor set)
        }
        crate::targets::gl::ext_data::append_stages!(sym > builder);
        bpx.write(builder)?;
    }
    Ok(())
}

fn write_packed_structs(bpx: &mut SymbolWriter<BufWriter<File>>, structs: Vec<StructOffset>, debug: bool) -> Result<(), Error>
{
    for sym in structs {
        //Unfortunately we must clone because rust is unable to see that sym.name is not used by
        // to_bpx_object...
        let mut builder = shader::symbol::Builder::new(sym.name.clone());
        builder
            .ty(shader::symbol::Type::ConstantBuffer)
            .internal()
            .extended_data(sym.to_bpx_object(debug, bpx)?);
        bpx.write(builder)?;
    }
    Ok(())
}

fn write_cbuffers(bpx: &mut SymbolWriter<BufWriter<File>>, objects: Vec<Object<StructOffset>>, debug: bool) -> Result<(), Error>
{
    for sym in objects {
        //Unfortunately we must clone because rust is unable to see that sym.inner.inner.name is
        // not used by to_bpx_object...
        let mut builder = shader::symbol::Builder::new(sym.inner.inner.name.clone());
        let slot = sym.inner.slot.get();
        if slot > 32 {
            error!("OpenGL limits texture/sampler bindings to 32, got a binding at register {}", slot);
            return Err(Error::new("unsupported binding register number"));
        } else if slot > 16 {
            warn!("This shader needs more than 16 bindings, this may not work on all hardware");
        }
        builder
            .register(slot as _)
            .ty(shader::symbol::Type::ConstantBuffer)
            .extended_data(sym.inner.inner.to_bpx_object(debug, bpx)?);
        if sym.inner.explicit.get() {
            builder.external();
        } else {
            builder.internal();
        }
        crate::targets::gl::ext_data::append_stages!(sym > builder);
        bpx.write(builder)?;
    }
    Ok(())
}

fn write_vformat(bpx: &mut SymbolWriter<BufWriter<File>>, vformat: Option<Struct>, debug: bool) -> Result<(), Error>
{
    if let Some(sym) = vformat {
        //Unfortunately we must clone because rust is unable to see that sym.name is
        // not used by to_bpx_object...
        let mut builder = shader::symbol::Builder::new(sym.name.clone());
        builder
            .external()
            .ty(shader::symbol::Type::VertexFormat)
            .extended_data(sym.to_bpx_object(debug, &())?);
        bpx.write(builder)?;
    } else {
        warn!("No vertex format was found in shader pack build");
    }
    Ok(())
}

fn write_pipeline(bpx: &mut SymbolWriter<BufWriter<File>>, pipeline: Option<PipelineStatement>, debug: bool) -> Result<(), Error>
{
    if let Some(sym) = pipeline {
        //Unfortunately we must clone because rust is unable to see that sym.name is
        // not used by to_bpx_object...
        let mut builder = shader::symbol::Builder::new(sym.name.clone());
        builder
            .internal()
            .ty(shader::symbol::Type::Pipeline)
            .extended_data(sym.to_bpx_object(debug, &())?);
        bpx.write(builder)?;
    } else {
        warn!("No pipeline was found in shader pack build");
    }
    Ok(())
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

fn write_outputs(bpx: &mut SymbolWriter<BufWriter<File>>, outputs: Vec<Slot<Property>>, blendfuncs: Vec<BlendfuncStatement>, debug: bool) -> Result<(), Error>
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
                    return Err(Error::new("illegal render target output type"));
                }
            }
        };
        let mut builder = shader::symbol::Builder::new(sym.inner.pname);
        builder
            .internal()
            .ty(shader::symbol::Type::Output)
            .register(sym.slot.get() as _)
            .extended_data(output.to_bpx_object(debug, &())?);
        bpx.write(builder)?;
    }
    Ok(())
}

fn write_root_constants(bpx: &mut SymbolWriter<BufWriter<File>>, root_constants_layout: StructOffset, debug: bool) -> Result<(), Error>
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
                    return Err(Error::new("illegal constant type"));
                }
            }
        };
        builder.extended_data(obj.to_bpx_object(debug, &())?);
        bpx.write(builder)?;
    }
    Ok(())
}

pub fn write_bpx(bpx: ShaderPack<BufWriter<File>>, syms: Symbols, shaders: Vec<ShaderBytes>, debug: bool) -> Result<(), Error>
{
    /*let mut bpx = ShaderPack::create(BufWriter::new(File::create(path)?),
                                     shader::Builder::new()
                                         .ty(shader::Type::Pipeline)
                                         .target(tg));*/
    let mut writer = SymbolWriter::new(bpx);
    write_objects(&mut writer, syms.objects, debug)?;
    write_packed_structs(&mut writer, syms.packed_structs, debug)?;
    write_cbuffers(&mut writer, syms.cbuffers, debug)?;
    write_vformat(&mut writer, syms.vformat, debug)?;
    write_pipeline(&mut writer, syms.pipeline, debug)?;
    write_outputs(&mut writer, syms.outputs, syms.blendfuncs, debug)?;
    write_root_constants(&mut writer, syms.root_constant_layout, debug)?;
    let mut bpx = writer.into_inner();
    let mut tbl = bpx.shaders_mut();
    for stage in shaders {
        tbl.create(shader::Shader {
            stage: stage.stage,
            data: stage.data
        })?;
    }
    bpx.save()?;
    Ok(())
}

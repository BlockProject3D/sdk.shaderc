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

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use sal::ast::tree::{PipelineStatement, Property, PropertyType, Struct};
use crate::options::{Args, Error};
use crate::targets::basic::ext_data::{SymbolWriter, ToObject};
use crate::targets::gl::core::{Object, ShaderData1, Symbols};
use bpx::shader;
use bpx::shader::{ShaderPack, Stage};
use log::{error, warn};
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
        if let Some(val) = sym.inner.inner.ptype.to_bpx_object(debug, &())? {
            builder.extended_data(val);
        }
        if sym.inner.explicit.get() {
            builder.external(); //Global binding (goes in the global descriptor set)
        } else {
            builder.internal(); //Local binding (goes in the local descriptor set)
        }
        crate::targets::basic::ext_data::append_stages!(sym > builder);
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
        builder.ty(shader::symbol::Type::ConstantBuffer).internal();
        if let Some(obj) = sym.to_bpx_object(debug, bpx)? {
            builder.extended_data(obj);
        }
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
        builder.register(slot as _).ty(shader::symbol::Type::ConstantBuffer);
        if sym.inner.explicit.get() {
            builder.external();
        } else {
            builder.internal();
        }
        if let Some(obj) = sym.inner.inner.to_bpx_object(debug, bpx)? {
            builder.extended_data(obj);
        }
        crate::targets::basic::ext_data::append_stages!(sym > builder);
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
        builder.external().ty(shader::symbol::Type::VertexFormat);
        if let Some(obj) = sym.to_bpx_object(debug, &())? {
            builder.extended_data(obj);
        }
        bpx.write(builder)?;
    } else {
        warn!("No vertex format was found in shader pack build");
    }
    Ok(())
}

fn write_pipeline(bpx: &mut SymbolWriter<BufWriter<File>>, pipeline: Option<PipelineStatement>, debug: bool) -> Result<(), Error>
{
    if let Some(sym) = pipeline {

    } else {
        warn!("No pipeline was found in shader pack build");
    }
    Ok(())
}

pub fn write_bpx(path: &Path, syms: Symbols, shaders: Vec<ShaderData1>, args: &Args) -> Result<(), Error>
{
    let mut bpx = ShaderPack::create(BufWriter::new(File::create(path)?),
                                     shader::Builder::new()
                                         .ty(shader::Type::Pipeline)
                                         .target(shader::Target::GL42));
    let mut writer = SymbolWriter::new(bpx);
    write_objects(&mut writer, syms.objects, args.debug)?;
    write_packed_structs(&mut writer, syms.packed_structs, args.debug)?;
    write_cbuffers(&mut writer, syms.cbuffers, args.debug)?;
    write_vformat(&mut writer, syms.vformat, args.debug)?;
    bpx = writer.into_inner();
    bpx.save()?;
    todo!()
}

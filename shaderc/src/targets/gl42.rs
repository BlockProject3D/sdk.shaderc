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
use bpx::sd::serde::EnumSize;
use bpx::shader;
use bpx::shader::{ShaderPack, Stage};
use log::{error, info, warn};
use sal::ast::tree::{Property, PropertyType, TextureType};
use crate::options::{Args, Error};
use crate::targets::basic::{decompose_pass, merge_stages, test_symbols};
use crate::targets::gl::{compile_stages, EnvInfo, gl_relocate_bindings, gl_test_bindings, link_shaders, Object, ShaderData1, Symbols};
use serde::Deserialize;
use serde::Serialize;
use crate::targets::layout140::StructOffset;

#[derive(Deserialize, Serialize)]
enum TextureObjectType
{
    T3D,
    T2D,
    T2DArray,
    TCube
}

#[derive(Deserialize, Serialize)]
struct TextureObject
{
    pub ty: TextureObjectType,
    pub value: TextureType
}

pub fn write_objects(bpx: &mut ShaderPack<BufWriter<File>>, objects: Vec<Object<Property>>, debug: bool) -> Result<(), Error>
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
        let st = match sym.inner.inner.ptype {
            PropertyType::Texture2D(value) => Some(TextureObject {
                ty: TextureObjectType::T2D,
                value
            }),
            PropertyType::Texture3D(value) => Some(TextureObject {
                ty: TextureObjectType::T3D,
                value
            }),
            PropertyType::Texture2DArray(value) => Some(TextureObject {
                ty: TextureObjectType::T2DArray,
                value
            }),
            PropertyType::TextureCube(value) => Some(TextureObject {
                ty: TextureObjectType::TCube,
                value
            }),
            _ => None
        };
        if let Some(st) = st {
            let val: bpx::sd::Object = st.serialize(bpx::sd::serde::Serializer::new(EnumSize::U8, debug))?.try_into().unwrap();
            builder.extended_data(val);
        }
        if sym.inner.explicit.get() {
            builder.external();
        } else {
            builder.internal();
        }
        //More code duplication, well say thanks to rust move semantics, you want performance, then code duplication!
        //TODO: If there's any workaround...
        if sym.stage_pixel {
            builder.stage(Stage::Pixel);
        }
        if sym.stage_domain {
            builder.stage(Stage::Domain);
        }
        if sym.stage_hull {
            builder.stage(Stage::Hull);
        }
        if sym.stage_vertex {
            builder.stage(Stage::Vertex);
        }
        if sym.stage_geometry {
            builder.stage(Stage::Geometry);
        }
        bpx.add_symbol(builder)?;
    }
    Ok(())
}

pub fn write_cbuffers(bpx: &mut ShaderPack<BufWriter<File>>, objects: Vec<Object<StructOffset>>, debug: bool) -> Result<(), Error>
{
    for sym in objects {
        let mut builder = shader::symbol::Builder::new(sym.inner.inner.name);
        let slot = sym.inner.slot.get();
        if slot > 32 {
            error!("OpenGL limits texture/sampler bindings to 32, got a binding at register {}", slot);
            return Err(Error::new("unsupported binding register number"));
        } else if slot > 16 {
            warn!("This shader needs more than 16 bindings, this may not work on all hardware");
        }
        builder.register(slot as _);
        if sym.inner.explicit.get() {
            builder.external();
        } else {
            builder.internal();
        }
        //TODO: generate extended data for constant buffers
        //More code duplication, well say thanks to rust move semantics, you want performance, then code duplication!
        //TODO: If there's any workaround...
        if sym.stage_pixel {
            builder.stage(Stage::Pixel);
        }
        if sym.stage_domain {
            builder.stage(Stage::Domain);
        }
        if sym.stage_hull {
            builder.stage(Stage::Hull);
        }
        if sym.stage_vertex {
            builder.stage(Stage::Vertex);
        }
        if sym.stage_geometry {
            builder.stage(Stage::Geometry);
        }
        bpx.add_symbol(builder)?;
    }
    Ok(())
}

fn write_bpx(path: &Path, syms: Symbols, shaders: Vec<ShaderData1>, args: &Args) -> Result<(), Error>
{
    let mut bpx = ShaderPack::create(BufWriter::new(File::create(path)?),
                                     shader::Builder::new()
                                         .ty(shader::Type::Pipeline)
                                         .target(shader::Target::GL42));
    write_objects(&mut bpx, syms.objects, args.debug)?;
    write_cbuffers(&mut bpx, syms.cbuffers, args.debug)?;
    todo!()
}

pub fn build(args: Args) -> Result<(), Error>
{
    info!("Running initial shader decomposition phase...");
    let shaders = decompose_pass(&args)?;
    info!("Merging shader stages");
    let mut stages = merge_stages(shaders)?;
    info!("Testing SAL symbols...");
    test_symbols(&stages)?;
    info!("Applying binding relocations...");
    gl_relocate_bindings(&mut stages);
    info!("Testing binding relocations...");
    gl_test_bindings(&stages)?;
    let (syms, shaders) = rglslang::main(|| {
        let env = EnvInfo {
            gl_version_int: 420,
            gl_version_str: "4.2",
            explicit_bindings: true
        };
        info!("Compiling shaders...");
        let output = compile_stages(&env, &args, stages)?; //We have a problem rust does not allow passing the error back to the build function
        info!("Linking shaders...");
        link_shaders(&args, output)
    })?;
    info!("Writing {}...", args.output.display());
    write_bpx(args.output, syms, shaders, &args)?;
    todo!()
}

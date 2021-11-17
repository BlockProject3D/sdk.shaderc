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

use std::{fs::File, io::BufWriter};

use bpx::package::{utils::pack_file_vname, Architecture, PackageBuilder, Platform};
use log::warn;

use crate::{
    options::{Error, ShaderUnit},
    targets::basic::shaderlib::ShaderLib
};
use crate::options::Args;

pub fn build(args: Args) -> Result<(), Error>
{
    let mut libs: Vec<ShaderLib> = args.libs.iter().map(|v| ShaderLib::new(*v)).collect();
    let mut bpxp = PackageBuilder::new()
        .with_type(*b"SL") //SL for ShaderLib
        .with_architecture(Architecture::Any)
        .with_platform(Platform::Any)
        .build(BufWriter::new(File::create(args.output)?))?;
    for unit in args.units {
        match unit {
            ShaderUnit::Path(path) => {
                if let Some(name) = path.file_name() {
                    if let Some(vname) = name.to_str() {
                        pack_file_vname(&mut bpxp, vname, path)?;
                    } else {
                        warn!(
                            "Path '{}' does not contain a valid file name, skipping...",
                            path.display()
                        );
                        continue;
                    }
                } else {
                    warn!(
                        "Path '{}' does not contain a valid file name, skipping...",
                        path.display()
                    );
                    continue;
                }
            },
            ShaderUnit::Injected(vname) => {
                for v in &mut libs {
                    if let Some(data) = v.try_load(vname)? {
                        bpxp.pack_object(vname, data.as_slice())?;
                    }
                }
            },
        }
    }
    bpxp.save()?;
    Ok(())
}

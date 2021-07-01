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

use bpx::encoder::Encoder;
use std::path::Path;
use std::fs::File;
use bpx::variant::package::PackageBuilder;
use crate::sal::compiler::ObjectType;
use bpx::sd::Object;
use std::vec::Vec;
use std::collections::HashSet;
use std::string::String;

pub enum Error
{
    Io(std::io::Error),
    Bpx(bpx::error::Error),
    Link(String)
}

impl From<std::io::Error> for Error
{
    fn from(e: std::io::Error) -> Self {
        return Error::Io(e);
    }
}

impl From<bpx::error::Error> for Error
{
    fn from(e: bpx::error::Error) -> Self {
        return Error::Bpx(e);
    }
}

pub fn assemble(out: &Path, objects: Vec<(String, Object)>) -> Result<(), Error>
{
    let mut set: HashSet<&String> = HashSet::new();
    let mut bpx = Encoder::new(File::create(out)?)?;
    let mut bpxp = PackageBuilder::new().with_variant([0x53, 0x4F]).build(&mut bpx)?;
    for (o_name, o_data) in &objects {
        if set.contains(o_name) {
            return Err(Error::Link(format!("Multiple definitions of '{}'", &o_name)));
        }
        set.insert(&o_name);
        let mut bytebuf: Vec<u8> = Vec::new();
        o_data.write(&mut bytebuf)?;
        bpxp.pack_object(&o_name, &mut bytebuf.as_slice())?;
    }
    bpx.save()?;
    return Ok(());
}

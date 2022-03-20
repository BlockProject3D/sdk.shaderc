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

use std::{
    fs::File,
    io::BufReader,
    path::Path
};

use bpx::macros::impl_err_conversion;
use bpx::package::Package;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error
{
    #[error("io error: {0}")]
    Io(std::io::Error),

    #[error("bpx error: {0}")]
    Bpx(bpx::package::error::Error)
}

impl_err_conversion!(
    Error {
        std::io::Error => Io,
        bpx::package::error::Error => Bpx
    }
);

struct ShaderLibDecoder
{
    package: Package<BufReader<File>>
}

impl ShaderLibDecoder
{
    pub fn new(path: &Path) -> Result<ShaderLibDecoder, Error>
    {
        let file = File::open(path)?;
        let package = Package::open(BufReader::new(file))?;
        Ok(ShaderLibDecoder {
            package
        })
    }

    pub fn try_load(&self, name: &str) -> Result<Option<Vec<u8>>, Error>
    {
        let mut data = Vec::new();
        let objects = self.package.objects()?;
        if let Some(obj) = objects.find(name)? {
            objects.load(obj, &mut data)?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}

pub struct ShaderLib<'a>
{
    path: &'a Path,
    decoder: Option<ShaderLibDecoder>
}

impl<'a> ShaderLib<'a>
{
    pub fn new(path: &'a Path) -> Self
    {
        Self { path, decoder: None }
    }

    pub fn try_load(&mut self, name: &str) -> Result<Option<Vec<u8>>, Error>
    {
        if self.decoder.is_none() {
            self.decoder = Some(ShaderLibDecoder::new(self.path)?);
        }
        let val = unsafe { self.decoder.as_ref().unwrap_unchecked() };
        val.try_load(name)
    }
}

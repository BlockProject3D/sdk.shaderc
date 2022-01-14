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
    fmt::{Display, Formatter},
    fs::File,
    io::BufReader,
    path::Path
};

use bpx::macros::impl_err_conversion;
use bpx::package::Package;
use bpx::utils::OptionExtension;

#[derive(Debug)]
pub enum Error
{
    Io(std::io::Error),
    Bpx(bpx::package::error::ReadError),
    Strings(bpx::strings::ReadError)
}

impl_err_conversion!(
    Error {
        std::io::Error => Io,
        bpx::package::error::ReadError => Bpx,
        bpx::strings::ReadError => Strings
    }
);

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::Bpx(e) => write!(f, "bpx error: {}", e),
            Error::Strings(e) => write!(f, "strings error: {}", e)
        }
    }
}

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

    pub fn try_load(&mut self, name: &str) -> Result<Option<Vec<u8>>, Error>
    {
        let mut data = Vec::new();
        if let Some(_) = self.package.unpack(name, &mut data)? {
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
        let val = self.decoder.get_or_insert_with_err(|| ShaderLibDecoder::new(self.path))?;
        val.try_load(name)
    }
}

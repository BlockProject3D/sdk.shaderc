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

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use bpx::macros::impl_err_conversion;
use bpx::package::object::ObjectHeader;
use bpx::package::PackageDecoder;
use bpx::package::utils::unpack_memory;
use bpx::table::ItemTable;

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
            Error::Strings(e) => write!(f, "strings error: {}", e),
        }
    }
}

struct ShaderLibDecoder
{
    decoder: PackageDecoder<BufReader<File>>,
    items: ItemTable<ObjectHeader>
}

impl ShaderLibDecoder
{
    pub fn try_load(&mut self, name: &str) -> Result<Option<Vec<u8>>, Error>
    {
        if let Some(obj) = self.items.lookup(name) {
            let data = unpack_memory(&mut self.decoder, obj)?;
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
        Self {
            path,
            decoder: None
        }
    }

    pub fn try_load(&mut self, name: &str) -> Result<Option<Vec<u8>>, Error>
    {
        if self.decoder.is_none() {
            let mut decoder = PackageDecoder::new(BufReader::new(File::open(self.path)?))?;
            let (mut items, mut names) = decoder.read_object_table()?;
            items.build_lookup_table(&mut names)?;
            self.decoder = Some(ShaderLibDecoder {
                decoder,
                items
            });
        }
        self.decoder.as_mut().unwrap().try_load(name)
    }
}

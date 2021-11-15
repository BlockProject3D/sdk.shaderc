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

use bpx::decoder::IoBackend;
use bpx::macros::impl_err_conversion;
use bpx::package::object::ObjectHeader;
use bpx::package::PackageDecoder;
use bpx::package::utils::unpack_memory;
use bpx::table::{ItemTable, NameTable};
use bpx::utils::OptionExtension;

#[derive(Debug)]
pub enum Error
{
    Bpx(bpx::package::error::ReadError),
    Strings(bpx::strings::ReadError)
}

impl_err_conversion!(
    Error {
        bpx::package::error::ReadError => Bpx,
        bpx::strings::ReadError => Strings
    }
);

pub struct ShaderLib<TBackend: IoBackend>
{
    decoder: PackageDecoder<TBackend>,
    table: Option<(ItemTable<ObjectHeader>, NameTable<ObjectHeader>)>
}

impl<TBackend: IoBackend> ShaderLib<TBackend>
{
    pub fn new(decoder: PackageDecoder<TBackend>) -> Self
    {
        Self {
            decoder,
            table: None
        }
    }

    pub fn new_with_table(decoder: PackageDecoder<TBackend>, table: (ItemTable<ObjectHeader>, NameTable<ObjectHeader>)) -> Self
    {
        Self {
            decoder,
            table: Some(table)
        }
    }

    pub fn try_load(&mut self, name: &str) -> Result<Option<Vec<u8>>, Error>
    {
        let flag = self.table.is_none();
        let (items, names) = self.table.get_or_insert_with_err(|| self.decoder.read_object_table())?;
        if flag {
            items.build_lookup_table(names)?;
        }
        if let Some(obj) = items.lookup(name) {
            let data = unpack_memory(&mut self.decoder, obj)?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}

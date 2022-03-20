// Copyright (c) 2022, BlockProject 3D
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
use serde::Serialize;
use bp3d_symbols::{ConstantObject, OutputObject, PipelineObject, Refs, StructObject, TextureObject, ToBpx};

pub type ExtDataPtr = Box<dyn ExtData + Send + Sync>;

pub trait ExtData {
    fn clone_erase_refs(&self) -> ExtDataPtr; // Returns a cloned object erased of all symbol refs.
    fn rewrite_refs(&self, map: &HashMap<usize, usize>) -> ExtDataPtr;
    fn refs(&self) -> &[usize]; // Returns a list of symbol refs.
    fn to_binary(&self) -> bincode::Result<Vec<u8>>; // Computes a binary representation of this extended data.
    fn to_bpx(&self, debug: bool) -> Result<bpx::sd::Value, bpx::sd::serde::Error>;
}

pub trait IntoExtData {
    fn into_ext_data(self) -> ExtDataPtr;
}

macro_rules! impl_into_ext_data {
    ($($t: ty)*) => {
        $(
            impl IntoExtData for $t {
                fn into_ext_data(self) -> ExtDataPtr {
                    Box::new(ExtDataImpl(self))
                }
            }
        )*
    };
}

macro_rules! impl_into_ext_data_with_refs {
    ($($t: ty)*) => {
        $(
            impl IntoExtData for $t {
                fn into_ext_data(self) -> ExtDataPtr {
                    Box::new(ExtDataImplWithRefs{
                        refs: self.list_refs(),
                        inner: self
                    })
                }
            }
        )*
    };
}

impl_into_ext_data!(ConstantObject TextureObject OutputObject PipelineObject);
impl_into_ext_data_with_refs!(StructObject);

struct ExtDataImplWithRefs<T> {
    refs: Vec<usize>,
    inner: T
}

impl<T: 'static + Send + Sync + Serialize + Refs + ToBpx> ExtData for ExtDataImplWithRefs<T> {
    fn clone_erase_refs(&self) -> ExtDataPtr {
        Box::new(ExtDataImplWithRefs {
            inner: self.inner.rewrite_refs(|_| 0),
            refs: Vec::new()
        })
    }

    fn rewrite_refs(&self, map: &HashMap<usize, usize>) -> ExtDataPtr {
        Box::new(ExtDataImplWithRefs {
            inner: self.inner.rewrite_refs(|v| map[&v.into()] as u16),
            refs: Vec::new()
        })
    }

    fn refs(&self) -> &[usize] {
        &self.refs
    }

    fn to_binary(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self.inner)
    }

    fn to_bpx(&self, debug: bool) -> Result<bpx::sd::Value, bpx::sd::serde::Error> {
        self.inner.to_bpx(debug)
    }
}

struct ExtDataImpl<T>(T);

impl<T: 'static + Send + Sync + Clone + Serialize + ToBpx> ExtData for ExtDataImpl<T> {
    fn clone_erase_refs(&self) -> ExtDataPtr {
        Box::new(ExtDataImpl(self.0.clone()))
    }

    fn rewrite_refs(&self, _: &HashMap<usize, usize>) -> ExtDataPtr {
        self.clone_erase_refs()
    }

    fn refs(&self) -> &[usize] {
        &[]
    }

    fn to_binary(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self.0)
    }

    fn to_bpx(&self, debug: bool) -> Result<bpx::sd::Value, bpx::sd::serde::Error> {
        self.0.to_bpx(debug)
    }
}

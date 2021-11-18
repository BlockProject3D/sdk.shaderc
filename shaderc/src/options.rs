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

use std::{borrow::Cow, fmt::Display, path::Path};

#[derive(Debug)]
pub enum ShaderUnit<'a>
{
    Path(&'a Path),
    Injected(&'a str)
}

pub struct Args<'a>
{
    pub units: Vec<ShaderUnit<'a>>,
    pub libs: Vec<&'a Path>,
    pub output: &'a Path,
    pub n_threads: usize,
    pub minify: bool,
    pub optimize: bool,
    pub debug: bool
}

#[derive(Debug, Clone)]
pub struct Error
{
    msg: Cow<'static, str>
}

impl Error
{
    pub fn new(msg: &'static str) -> Self
    {
        Self { msg: msg.into() }
    }

    pub fn into_inner(self) -> Cow<'static, str>
    {
        self.msg
    }
}

impl<T: Display> From<T> for Error
{
    fn from(v: T) -> Self
    {
        Self {
            msg: format!("{}", v).into()
        }
    }
}

pub type TargetFunc = fn(Args) -> Result<(), Error>;

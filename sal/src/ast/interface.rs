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

use std::fmt::Debug;
use crate::ast::error::Error;
use crate::ast::tree::Statement;
use crate::parser::tree::Use;

pub trait IntoError<Ok, Err>
{
    fn into_error(self) -> Result<Ok, Err>;
}

pub trait UseResolver
{
    type Error: Debug;

    fn resolve(&mut self, item: Use) -> Result<Statement, Self::Error>;
}

impl<E: Debug> IntoError<Statement, Error<E>> for Result<Statement, E>
{
    fn into_error(self) -> Result<Statement, Error<E>>
    {
        self.map_err(|e| Error::UnresolvedUse(e))
    }
}

impl<T> UseResolver for &mut T where T: UseResolver
{
    type Error = T::Error;

    fn resolve(&mut self, item: Use) -> Result<Statement, Self::Error>
    {
        (**self).resolve(item)
    }
}

pub struct IgnoreUseResolver
{
}

impl UseResolver for IgnoreUseResolver
{
    type Error = ();

    fn resolve(&mut self, _: Use) -> Result<Statement, Self::Error>
    {
        Ok(Statement::Noop)
    }
}

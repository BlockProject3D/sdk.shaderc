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

use std::io::BufRead;

pub fn run<T: BufRead, Handler: crate::preprocessor::Handler>(reader: T, mut handler: Handler) -> Result<(), Handler::Error>
{
    let mut sal_block = false;

    for v in reader.lines() {
        let line = v?;
        let trimed = line.trim();
        if trimed == "#sal" {
            sal_block = !sal_block;
            handler.directive(trimed[1..].trim(), None)?;
        } else if sal_block {
            handler.sal_code(&line)?;
        } else if trimed.starts_with('#') {
            if let Some(id) = trimed.find(' ') {
                let name = trimed[1..id].trim();
                let value = trimed[id..].trim();
                handler.directive(name, Some(value))?;
            } else {
                let trimed = trimed[1..].trim();
                handler.directive(trimed, None)?;
            }
        }
        handler.code_line(line)?;
    }
    Ok(())
}

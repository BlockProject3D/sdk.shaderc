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

use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::Path;
use log::LevelFilter;

pub fn alloc_verbosity_level(verbosity: u64) {
    match verbosity {
        0 => log::set_max_level(LevelFilter::Error),
        1 => log::set_max_level(LevelFilter::Warn),
        2 => log::set_max_level(LevelFilter::Info),
        3 => log::set_max_level(LevelFilter::Debug),
        _ => log::set_max_level(LevelFilter::Trace),
    };
}

pub fn init_bp3d_logger<F: FnOnce() -> i32>(f: F) {
    //Initialize bp3d-logger
    let res = bp3d_logger::Logger::new().add_stdout().add_file("bp3d-sdk").run(f);
    std::process::exit(res);
}

fn transform_output(path: &Path) -> Cow<Path>
{
    if path.is_dir() {
        return path.join("a.out.bpx").into();
    }
    if path.extension().unwrap_or_default() != "bpx" {
        let mut path = path.to_owned();
        path.set_extension("bpx");
        path.into()
    } else {
        path.into()
    }
}

pub fn get_out_path(arg: Option<&OsStr>) -> Cow<Path> {
    transform_output(arg
        .map(|v| Path::new(v))
        .unwrap_or(Path::new("a.out.bpx"))
    )
}

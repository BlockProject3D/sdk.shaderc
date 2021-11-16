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

mod thread_pool;
mod targets;
mod options;

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use clap::{App, Arg, OsValues};
use log::{info, LevelFilter};
use crate::options::{Args, ShaderUnit};

//mod sal;

//pub use sal::Lexer;

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

fn main()
{
    //Log everything
    log::set_max_level(LevelFilter::Trace);
    info!("Initializing BlockProject 3D Shader Compiler...");
    let matches = App::new("shaderc")
        .author("BlockProject 3D")
        .about("BlockProject 3D SDK - Shader Compiler")
        .version("1.0.0")
        .args([
            Arg::new("target").short('t').long("--target").takes_value(true).required(true)
                .about("Specify the shader package target"),
            Arg::new("print_targets").long("--targets")
                .about("Print all available shader package targets"),
            Arg::new("output").short('o').long("output").takes_value(true)
                .about("Output shader package file name"),
            Arg::new("lib").short('l').long("lib").takes_value(true).multiple_occurrences(true)
                .about("Specify one or more shader libs to use"),
            Arg::new("injection").short('i').long("inject").takes_value(true).multiple_occurrences(true)
                .about("Inject a shader contained in one of the linked libs such that it will always be included in the compilation"),
            Arg::new("threads").short('n').long("threads").takes_value(true)
                .about("Specify the maximum number of threads to use when processing shaders"),
            Arg::new("minify").short('m').long("minify")
                .about("Allows minification of source code in targets that do not support actual compilation (ex: GL targets)"),
            Arg::new("debug").short('d').long("debug")
                .about("For supported targets, builds shaders with debug info"),
            Arg::new("optimize").short('O').long("optimize")
                .about("For supported targets, builds shaders with optimizations"),
            Arg::new("shader").multiple_values(true).about("List of shader files to process")
        ])
        .get_matches();
    /*let matches = clap_app!(shaderc =>
        (version: "1.0.0")
        (author: "BlockProject 3D")
        (about: "BlockProject 3D SDK - Shader Compiler")
        (@arg target: -t --target +takes_value +required "Specify the shader package target")
        (@arg print_targets: --targets "Print all available shader package targets")
        (@arg output: -o --output +takes_value "Output shader package file name")
        (@arg libs: -l --lib +takes_value +multiple "Specify one or more shader libs to use")
        (@arg injections: -i --inject +takes_value +multiple "Inject a shader contained in one of the linked libs such that it will always be included in the compilation")
        (@arg threads: -n --threads +takes_value "Specify the maximum number of threads to use when processing shaders")
        (@arg minify: -m --minify "Allows minification of source code in targets that do not support actual compilation (ex: GL targets)")
        (@arg debug: -d --debug "For supported targets, builds shaders with debug info")
        (@arg optimize: -O --optimize "For supported targets, builds shaders with optimizations")
        (@arg shaders: ... "List of shader files to process")
    ).get_matches();*/
    let mut units: Vec<ShaderUnit> = matches.values_of_os("shader").unwrap_or_default().map(|v| ShaderUnit::Path(Path::new(v))).collect();
    let libs: Vec<&Path> = matches.values_of_os("lib").unwrap_or_default().map(|v| Path::new(v)).collect();
    let n_threads: usize = matches.value_of_t("threads").unwrap_or(1);
    let minify = matches.is_present("minify");
    let optimize = matches.is_present("optimize");
    let debug = matches.is_present("debug");
    let output = transform_output(matches.value_of_os("output").map(|v| Path::new(v)).unwrap_or(Path::new("a.out.bpx")));
    for v in matches.values_of("injection").unwrap_or_default() {
        units.push(ShaderUnit::Injected(v));
    }
    let args = Args {
        units,
        libs,
        n_threads,
        minify,
        optimize,
        debug,
        output: output.as_ref()
    };
    let target = matches.value_of("target").unwrap();
    let args: Vec<OsString> = std::env::args_os().collect();
    println!("Hello, world! {:?}", args);
}

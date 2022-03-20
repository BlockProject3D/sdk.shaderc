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

mod options;
mod targets;
mod config;

use std::path::Path;

use clap::{Arg, Command};
use log::{debug, error, info};
use phf::phf_map;
use cli_common::{alloc_verbosity_level, get_out_path, init_bp3d_logger};

const PROG_NAME: &str = env!("CARGO_PKG_NAME");
const PROG_VERSION: &str = env!("CARGO_PKG_VERSION");

static TARGETS: phf::Map<&'static str, options::TargetFunc> = phf_map! {
    "LIB" => targets::lib::build,
    "GL40" => targets::gl40::build,
    "GL42" => targets::gl42::build
};

fn run() -> i32
{
    let matches = Command::new(PROG_NAME)
        .author("BlockProject 3D")
        .about("BlockProject 3D SDK - Shader Compiler")
        .version(PROG_VERSION)
        .args([
            Arg::new("verbose").short('v').long("verbose").multiple_occurrences(true)
                .help("Enable verbose output"),
            Arg::new("target").short('t').long("--target").takes_value(true).required_unless_present("print_targets")
                .help("Specify the shader package target"),
            Arg::new("print_targets").long("--targets")
                .help("Print all available shader package targets"),
            Arg::new("output").short('o').long("output").takes_value(true)
                .allow_invalid_utf8(true).help("Output shader package file name"),
            Arg::new("lib").short('l').long("lib").takes_value(true).multiple_occurrences(true)
                .allow_invalid_utf8(true).help("Specify one or more shader libs to use"),
            Arg::new("injection").short('i').long("inject").takes_value(true).multiple_occurrences(true)
                .help("Inject a shader contained in one of the linked libs such that it will always be included in the compilation"),
            Arg::new("threads").short('n').long("threads").takes_value(true)
                .help("Specify the maximum number of threads to use when processing shaders"),
            Arg::new("minify").short('m').long("minify")
                .help("Allows minification of source code in targets that do not support actual compilation (ex: GL targets)"),
            Arg::new("debug").short('d').long("debug")
                .help("For supported targets, builds shaders with debug info"),
            Arg::new("optimize").short('O').long("optimize")
                .help("For supported targets, builds shaders with optimizations"),
            Arg::new("shader").multiple_values(true).allow_invalid_utf8(true)
                .help("List of shader files to process")
        ]).get_matches();
    alloc_verbosity_level(matches.occurrences_of("verbose"));
    info!("Initializing BlockProject 3D Shader Compiler...");
    if matches.is_present("print_targets") {
        print!("Available targets: ");
        for (i, name) in TARGETS.keys().enumerate() {
            if i == TARGETS.len() - 1 {
                print!("{}", name)
            } else {
                print!("{}, ", name)
            }
        }
        println!();
        0
    } else {
        let mut units: Vec<config::Unit> = matches
            .values_of_os("shader")
            .unwrap_or_default()
            .map(|v| config::Unit::Path(Path::new(v)))
            .collect();
        let libs: Vec<&Path> = matches
            .values_of_os("lib")
            .unwrap_or_default()
            .map(|v| Path::new(v))
            .collect();
        let n_threads: usize = matches.value_of_t("threads").unwrap_or(1);
        let minify = matches.is_present("minify");
        let optimize = matches.is_present("optimize");
        let debug = matches.is_present("debug");
        let output = get_out_path(matches.value_of_os("output"));
        for v in matches.values_of("injection").unwrap_or_default() {
            units.push(config::Unit::Injected(v));
        }
        let config = config::Config {
            units,
            libs,
            n_threads,
            minify,
            optimize,
            debug,
            output: output.as_ref()
        };
        let target = matches.value_of("target").unwrap();
        debug!("Target chosen: {}", target);
        if let Some(func) = TARGETS.get(target) {
            info!("Building for target: {}...", target);
            if let Err(e) = func(config) {
                error!("{}", e);
                1
            } else {
                0
            }
        } else {
            error!("Target not found: {}", target);
            3
        }
    }
}

fn main() {
    init_bp3d_logger(run);
}

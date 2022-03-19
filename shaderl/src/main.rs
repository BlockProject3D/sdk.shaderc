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

mod symbols;
mod tree;
mod ext_data;
mod assembler;

use std::path::Path;
use clap::{Arg, ArgMatches, Command};
use log::{error, info};
use cli_common::{alloc_verbosity_level, get_out_path, init_bp3d_logger};

const PROG_NAME: &str = env!("CARGO_PKG_NAME");
const PROG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn assemble(n_threads: usize, args: &ArgMatches) -> i32 {
    let debug = args.is_present("debug");
    let output = get_out_path(args.value_of_os("output"));
    let assembly = args.value_of_os("assembly").map(Path::new);
    let name = args.value_of("name").unwrap();
    let shaders = args.values_of_os("shader")
        .unwrap_or_default()
        .map(Path::new);
    let cfg = assembler::Config {
        n_threads,
        debug,
        output: &output,
        assembly,
        name,
        shaders
    };
    if let Err(e) = assembler::run(cfg) {
        error!("{}", e);
        1
    } else {
        0
    }
}

fn run() -> i32 {
    let matches = Command::new(PROG_NAME)
        .author("BlockProject 3D")
        .about("BlockProject 3D SDK - Shader Linker")
        .version(PROG_VERSION)
        .subcommand_required(true)
        .subcommands([
            Command::new("link").about("Link shader pack(s) to a shader assembly")
                .args([
                    Arg::new("assembly").required(true).short('a').long("assembly")
                        .takes_value(true).allow_invalid_utf8(true)
                        .help("Path of the shader assembly to link to"),
                    Arg::new("shader").multiple_values(true).allow_invalid_utf8(true)
                        .help("List of shader pack(s) to link")
                ]),
            Command::new("assemble").about("Assemble a shader assembly from shader pack(s)")
                .args([
                    Arg::new("name").short('n').long("name")
                        .takes_value(true).required(true)
                        .help("Name of assembly"),
                    Arg::new("assembly").short('a').long("assembly")
                        .takes_value(true).allow_invalid_utf8(true)
                        .help("Path to a parent shader assembly"),
                    Arg::new("output").short('o').long("output").takes_value(true)
                        .allow_invalid_utf8(true).help("Output shader assembly file name"),
                    Arg::new("debug").short('d').long("debug")
                        .help("Build the shader assembly with debug info"),
                    Arg::new("shader").multiple_values(true).allow_invalid_utf8(true)
                        .help("List of shader pack(s) to assemble")
                ])
        ])
        .args([
            Arg::new("verbose").short('v').long("verbose").multiple_occurrences(true)
                .help("Enable verbose output"),
            Arg::new("threads").short('n').long("threads").takes_value(true)
                .help("Specify the maximum number of threads to use when processing shaders")
        ]).get_matches();
    alloc_verbosity_level(matches.occurrences_of("verbose"));
    info!("Initializing BlockProject 3D Shader Linker...");
    let n_threads: usize = matches.value_of_t("threads").unwrap_or(1);
    if let Some(args) = matches.subcommand_matches("assemble") {
        return assemble(n_threads, args);
    }
    /*if let Some(args) = matches.subcommand_matches("link") {

    }*/
    0
}

fn main() {
    init_bp3d_logger(run);
}

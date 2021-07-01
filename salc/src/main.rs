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

use std::{
    path::{Path, PathBuf},
    vec::Vec
};

use clap::clap_app;

use crate::error::Error;
use bpx::sd::Object;

mod assembler;
mod error;
mod preprocessor;
mod sal;

fn run_salc(input: &Path, output: &Path, module_paths: Vec<PathBuf>) -> Result<(), error::Error>
{
    //Stage 1: apply preprocessor
    let shader = preprocessor::run(input)?;
    //Stage 2: apply lexer to SAL code
    let mut lexer = sal::lexer::Lexer::new();
    for v in &shader.sal_code {
        if let Err(e) = lexer.push_str(&v) {
            return Err(error::Error::Lexer(e));
        }
    }
    //Stage 3: parse SAL code and reconstruct AST
    let statements = match sal::parse(lexer, true, &module_paths) {
        Err(e) => return Err(error::Error::Parser(e)),
        Ok(v) => v
    };
    //Stage 4: compile AST to lower level object code
    let objects = sal::compiler::compile(statements);
    //Stage 5: assemble output BPX
    let mut obj = Object::new();
    obj.set("Stage", (shader.stage as u8).into());
    assembler::assemble(output, objects, shader.shader_code, obj)?;
    return Ok(());
}

fn main()
{
    let matches = clap_app!(salc =>
        (version: "1.0.0")
        (author: "BlockProject 3D")
        (about: "BlockProject 3D SDK - SAL compiler")
        (@arg input: -i --input +takes_value +required "Input shader file name")
        (@arg output: -o --output +takes_value "Output file name")
        (@arg includes: -I --include +takes_value +multiple "Path to a directory to use to find SAL modules")
    )
    .get_matches();
    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output");
    let includes = matches.values_of("includes");
    let mut module_paths: Vec<PathBuf> = Vec::new();
    if let Some(v) = includes {
        for vv in v {
            module_paths.push(PathBuf::from(vv));
        }
    }
    let path;
    if let Some(s) = output {
        path = PathBuf::from(s);
    } else {
        let mut s = String::from(input);
        s.push_str(".o.bpx");
        path = PathBuf::from(s);
    }
    if let Err(e) = run_salc(Path::new(input), &path, module_paths) {
        match e {
            Error::Io(e) => eprintln!("Io error: {}", e),
            Error::Bpx(e) => eprintln!("BPX error: {}", e),
            Error::Link(e) => eprintln!("Link error: {}", e),
            Error::Lexer(e) => eprintln!("Lexer error: {}", e),
            Error::Parser(e) => eprintln!("Parse error: {}", e)
        }
        std::process::exit(1);
    }
    println!("Wrote '{}'", path.display());
    std::process::exit(0);
}

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

use std::ffi::OsString;

use clap::clap_app;
use rglslang::{
    environment::{Client, Stage},
    shader::{Messages, Part, Profile}
};

//mod sal;

//pub use sal::Lexer;

fn main()
{
    rglslang::main(|| {
        let shader = rglslang::shader::Builder::new(rglslang::environment::Environment::new_opengl(
            Stage::Vertex,
            Client::OpenGL,
            None
        ))
        .default_profile(Profile::Core)
        .default_version(330)
        .force_default_version_and_profile()
        .entry_point("main")
        .source_entry_point("main")
        .messages(Messages::new().debug().ast())
        .add_part(Part::new_with_name(
            std::fs::read_to_string("./shaderc/shader_file.glsl").unwrap(),
            "My shader"
        ))
        .parse();
        println!(
            "OK {}\n\n Info log\n{}\n\n Debug log\n{}",
            shader.check(),
            shader.get_info_log(),
            shader.get_info_debug_log()
        );
    });
    let matches = clap_app!(shaderc =>
        (version: "1.0.0")
        (author: "BlockProject 3D")
        (about: "BlockProject 3D SDK - shader compiler")
        (@arg input: -i --input +takes_value +required "Input shader file name")
        (@arg target: -t --target +takes_value +required "Shader target (GLSL, HLSL or VGLSL)")
        (@arg output: -o --output +takes_value "Output file name")
        (@arg includes: -I --include +takes_value +multiple "Path to a directory to use as includes for SAL or shader code")
    ).get_matches();
    let args: Vec<OsString> = std::env::args_os().collect();
    println!("Hello, world! {:?}", args);
}

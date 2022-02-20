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
    fs::File,
    io::{BufRead, BufReader},
    path::Path
};

use cc::{Build, Tool};
use regex::Regex;

struct Version
{
    minor: String,
    major: String,
    patch: String,
    flavor: String
}

fn parse_version(proj: &Path) -> Version
{
    let re = Regex::new(r"#+ *([0-9]+)\.([0-9]+)\.([0-9]+)(-[a-zA-Z0-9]+)?").expect("Failed to compile regex");
    let file = File::open(proj.join("CHANGES.md")).expect("Failed to open CHANGES.md file");
    let reader = BufReader::new(file);
    for res in reader.lines() {
        let line = res.expect("Failed to read line");
        if let Some(res) = re.captures(&line) {
            let major = &res[1];
            let minor = &res[2];
            let patch = &res[3];
            let flavor = match res.get(4) {
                Some(s) => s.as_str().into(),
                None => String::new()
            };
            return Version {
                minor: minor.into(),
                major: major.into(),
                patch: patch.into(),
                flavor
            };
        }
    }
    panic!("Unable to parse version");
}

fn generate_build_info_h(proj: &Path, out_dir: &Path)
{
    let dir = out_dir.join("glslang");
    let mut file =
        std::fs::read_to_string(proj.join("build_info.h.tmpl")).expect("Failed to read build_info.h template");
    let version = parse_version(proj);
    let out = dir.join("build_info.h");

    std::fs::create_dir_all(dir).expect("Failed to create glslang generated include directory");
    file = file.replace("@major@", &version.major);
    file = file.replace("@minor@", &version.minor);
    file = file.replace("@patch@", &version.patch);
    file = file.replace("@flavor@", &version.flavor);
    std::fs::write(&out, file).expect("Failed to write generated build_info.h");
}

fn append_files(builder: &mut Build, base_dir: &Path, sources: &[&str])
{
    for source in sources {
        builder.file(base_dir.join(source));
    }
}

fn build_glslang(proj: &Path, builder: &mut Build, compiler: &Tool)
{
    let os = std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap();
    let root = proj.join("glslang");
    if os == "windows" {
        builder.file(root.join("OSDependent/Windows/ossource.cpp"));
        if compiler.is_like_gnu() {
            builder.flag("-fpermissive");
        }
    } else if os == "unix" {
        builder.file(root.join("OSDependent/Unix/ossource.cpp"));
    }
    const GENERIC_CODE_GEN: &[&str] = &["GenericCodeGen/CodeGen.cpp", "GenericCodeGen/Link.cpp"];
    const MACHINE_INDEPENDENT: &[&str] = &[
        "MachineIndependent/glslang_tab.cpp",
        "MachineIndependent/attribute.cpp",
        "MachineIndependent/Constant.cpp",
        "MachineIndependent/iomapper.cpp",
        "MachineIndependent/InfoSink.cpp",
        "MachineIndependent/Initialize.cpp",
        "MachineIndependent/IntermTraverse.cpp",
        "MachineIndependent/Intermediate.cpp",
        "MachineIndependent/ParseContextBase.cpp",
        "MachineIndependent/ParseHelper.cpp",
        "MachineIndependent/PoolAlloc.cpp",
        "MachineIndependent/RemoveTree.cpp",
        "MachineIndependent/Scan.cpp",
        "MachineIndependent/ShaderLang.cpp",
        "MachineIndependent/SpirvIntrinsics.cpp",
        "MachineIndependent/SymbolTable.cpp",
        "MachineIndependent/Versions.cpp",
        "MachineIndependent/intermOut.cpp",
        "MachineIndependent/limits.cpp",
        "MachineIndependent/linkValidate.cpp",
        "MachineIndependent/parseConst.cpp",
        "MachineIndependent/reflection.cpp",
        "MachineIndependent/preprocessor/Pp.cpp",
        "MachineIndependent/preprocessor/PpAtom.cpp",
        "MachineIndependent/preprocessor/PpContext.cpp",
        "MachineIndependent/preprocessor/PpScanner.cpp",
        "MachineIndependent/preprocessor/PpTokens.cpp",
        "MachineIndependent/propagateNoContraction.cpp"
    ];
    const GLSLLANG_SOURCES: &[&str] = &["CInterface/glslang_c_interface.cpp"];
    append_files(builder, &root, GENERIC_CODE_GEN);
    append_files(builder, &root, MACHINE_INDEPENDENT);
    append_files(builder, &root, GLSLLANG_SOURCES);
}

fn build_ogl(proj: &Path, builder: &mut Build)
{
    let root = proj.join("OGLCompilersDLL");
    builder.file(root.join("InitializeDll.cpp"));
}

fn build_spirv(proj: &Path, builder: &mut Build)
{
    let root = proj.join("SPIRV");
    const SOURCES: &[&str] = &[
        "GlslangToSpv.cpp",
        "InReadableOrder.cpp",
        "Logger.cpp",
        "SpvBuilder.cpp",
        "SpvPostProcess.cpp",
        "doc.cpp",
        "disassemble.cpp",
        "CInterface/spirv_c_interface.cpp"
    ];
    append_files(builder, &root, SOURCES);
}

fn main()
{
    let proj = Path::new("./glslang");
    let useless = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&useless);
    let generated_include_dir = out_dir.join("include");
    generate_build_info_h(proj, &generated_include_dir);
    let mut builder = cc::Build::new();
    let compiler = builder.get_compiler();
    //Apparently cc crate is defective and all C++ code refuses to compile
    builder.flag("-std=c++14");
    builder.cpp(true);
    builder.warnings(false);
    if compiler.is_like_gnu() || compiler.is_like_clang() {
        builder.flag("-fno-exceptions");
        builder.flag("-fno-rtti");
    } else if compiler.is_like_msvc() {
        builder.flag("/GR-");
    }
    builder.include(&generated_include_dir);
    builder.include(&proj);
    build_glslang(proj, &mut builder, &compiler);
    build_ogl(proj, &mut builder);
    build_spirv(proj, &mut builder);
    //Add part of standalone EXE in order to get DefaultTBuiltinResource
    builder.file(proj.join("StandAlone/ResourceLimits.cpp"));
    // Add glue cpp
    builder.file(Path::new("./src/glue.cpp"));
    builder.compile("glslang");
    println!("cargo:rerun-if-changed=./glslang");
    println!("cargo:rerun-if-changed=./src/glue.cpp");
}

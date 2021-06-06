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

//Module for parsing Shader Annotation Language (SAL)

use std::{
    path::{Path, PathBuf},
    string::String
};

pub mod ast;
pub mod generator;
pub mod lexer;
pub mod parser;

pub fn parse(lexer: lexer::Lexer, expand_use: bool, module_paths: &Vec<PathBuf>)
    -> Result<Vec<ast::Statement>, String>
{
    let mut parser = parser::Parser::new(lexer);
    let tree = parser.parse()?;
    return generator::build_ast(tree, expand_use, module_paths);
}

pub fn parse_file(file: &Path, expand_use: bool, module_paths: &Vec<PathBuf>) -> Result<Vec<ast::Statement>, String>
{
    let str = match std::fs::read_to_string(file) {
        Err(e) => return Err(format!("Error loading SAL script file: {}", e)),
        Ok(v) => v
    };
    let mut lexer = lexer::Lexer::new();
    lexer.push_str(&str)?;
    return parse(lexer, expand_use, module_paths);
}

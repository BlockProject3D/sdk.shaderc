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

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;
use bpx::shader::Stage;
use log::warn;
use sal::ast::tree::Statement;
use sal::utils::auto_lexer_parser;
use crate::options::{Args, Error, ShaderUnit};
use crate::targets::basic::preprocessor::BasicPreprocessor;
use crate::targets::basic::shaderlib::ShaderLib;
use crate::targets::basic::useresolver::BasicUseResolver;
use sal::preprocessor;

pub struct ShaderToSal
{
    pub strings: Vec<rglslang::shader::Part>,
    pub statements: Vec<Statement>
}

fn shader_sal_stage<T: BufRead>(name: String, content: T, args: &Args) -> Result<ShaderToSal, Error>
{
    let mut result = ShaderToSal {
        strings: Vec::new(),
        statements: Vec::new()
    };
    let mut preprocessor = BasicPreprocessor::new(&args.libs);
    preprocessor::run(content, &mut preprocessor)?;
    let stage = preprocessor.stage.unwrap_or_else(|| {
        warn!("No shader stage specified in shader file, assuming this is a vertex shader by default");
        Stage::Vertex
    });
    for (name, header) in preprocessor.includes {
        let data = shader_sal_stage(name,header.deref(), args)?;
        result.strings.extend(data.strings);
        result.statements.extend(data.statements);
    }
    result.statements.extend(auto_lexer_parser(&preprocessor.sal_code, BasicUseResolver::new(&args.libs))?);
    result.strings.push(rglslang::shader::Part::new_with_name(preprocessor.src_code.join("\n"), name));
    Ok(result)
}

pub fn load_shader_to_sal(unit: &ShaderUnit, args: &Args) -> Result<ShaderToSal, Error>
{
    let mut libs: Vec<ShaderLib> = args.libs.iter().map(|v| ShaderLib::new(*v)).collect();
    match unit {
        ShaderUnit::Path(path) => {
            let reader = BufReader::new(File::open(path)?);
            shader_sal_stage(path.to_string_lossy().into_owned(),reader, args)
        },
        ShaderUnit::Injected(vname) => {
            for v in &mut libs {
                if let Some(data) = v.try_load(vname)? {
                    return shader_sal_stage(String::from(*vname), data.as_slice(), args);
                }
            }
            Err(Error::from(format!("unable to locate injected shader '{}'", vname)))
        }
    }
}

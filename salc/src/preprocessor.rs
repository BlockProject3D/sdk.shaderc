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

use std::string::String;
use std::vec::Vec;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use std::io;
use std::fs::File;
use phf::phf_map;

#[derive(Clone, Copy)]
pub enum ShaderStage
{
    Vertex,
    Pixel,
    Geometry,
    Hull,
    Domain,
    Unspecified
}

static SHADERSTAGE: phf::Map<&'static str, ShaderStage> = phf_map!
{
    "vertex" => ShaderStage::Vertex,
    "pixel" => ShaderStage::Pixel,
    "geometry" => ShaderStage::Geometry,
    "hull" => ShaderStage::Hull,
    "domain" => ShaderStage::Domain
};

pub struct ShaderObject
{
    stage: ShaderStage,
    sal_code: Vec<String>,
    shader_code: Vec<String>
}

pub fn run(file: &Path) -> io::Result<ShaderObject>
{
    let f = File::open(file)?;
    let reader = BufReader::new(f);
    let mut sstage = ShaderStage::Unspecified;
    let mut sal_block = false;
    let mut sal_code = Vec::new();
    let mut shader_code = Vec::new();

    for v in reader.lines()
    {
        let line = v?;
        let trimed = line.trim();
        if trimed.starts_with("#stage")
        {
            if let Some(id) = trimed.find(' ')
            {
                let stage = &trimed[id..].trim();
                if let Some(stage) = SHADERSTAGE.get(*stage)
                {
                    sstage = *stage;
                }
            }
        }
        else if trimed == "#sal"
        {
            sal_block = !sal_block;
        }
        else if sal_block
        {
            sal_code.push(line);
        }
        else
        {
            shader_code.push(line);
        }
    }
    return Ok(ShaderObject
    {
        stage: sstage,
        sal_code: sal_code,
        shader_code: shader_code
    });
}

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

use std::{collections::VecDeque, fmt::Display, string::String};

use regex::Regex;
use crate::token::{STR_COMMENT, Token};
use crate::token::{
    STR_CONST,
    STR_STRUCT,
    STR_PIPELINE,
    STR_VFORMAT,
    STR_USE,
    STR_EQ,
    STR_BLOCK_START,
    STR_BLOCK_END,
    STR_OUTPUT,
    STR_BREAK
};

fn check_keyword(substr: &str) -> Option<Token>
{
    return match substr {
        STR_CONST => Some(Token::Const),
        STR_STRUCT => Some(Token::Struct),
        STR_PIPELINE => Some(Token::Pipeline),
        STR_VFORMAT => Some(Token::Vformat),
        STR_USE => Some(Token::Use),
        STR_EQ => Some(Token::Eq),
        STR_BLOCK_START => Some(Token::BlockStart),
        STR_BLOCK_END => Some(Token::BlockEnd),
        STR_OUTPUT => Some(Token::Output),
        _ => None
    };
}

fn check_litteral(substr: &str) -> Option<Token>
{
    if substr == "true" {
        return Some(Token::Bool(true));
    } else if substr == "false" {
        return Some(Token::Bool(false));
    }
    if let Ok(v) = substr.parse::<i32>() {
        return Some(Token::Int(v));
    }
    if let Ok(v) = substr.parse::<f32>() {
        return Some(Token::Float(v));
    }
    return None;
}

fn check_identifier(substr: &str) -> Option<Token>
{
    let re = Regex::new(r"[A-z]([A-z]|\d)*").unwrap();
    if re.replace(substr, "") == "" {
        return Some(Token::Identifier(String::from(substr)));
    }
    return None;
}

fn check_namespace(substr: &str) -> Option<Token>
{
    let re = Regex::new(r"[A-z]([A-z]|\d)*::([A-z]|\d)+").unwrap();
    if re.replace(substr, "") == "" {
        return Some(Token::Namespace(String::from(substr)));
    }
    return None;
}

fn is_whitespace(substr: &str) -> bool
{
    return substr == " " || substr == "\t" || substr == "\r" || substr == "\n" || substr == STR_BREAK;
}

fn trim_token(code: &str, token: (usize, usize)) -> (usize, usize)
{
    let (mut pos1, mut pos2) = token;
    while pos1 < pos2 && is_whitespace(&code[pos1..pos1 + 1]) {
        pos1 += 1;
    }
    while pos2 > pos1 && is_whitespace(&code[pos2 - 1..pos2]) {
        pos2 -= 1;
    }
    return (pos1, pos2);
}

pub struct Lexer
{
    tokens: VecDeque<(Token, usize, usize)>,
    cur_token: (usize, usize),
    cur_line: usize,
    cur_column: usize,
    in_comment: bool
}

impl Lexer
{
    pub fn new() -> Lexer
    {
        return Lexer {
            tokens: VecDeque::new(),
            cur_token: (0, 0),
            cur_column: 0,
            cur_line: 1,
            in_comment: false
        };
    }

    fn parse_token(&mut self, pos1: usize, pos2: usize, code: &str) -> Result<(), String>
    {
        let (np1, np2) = trim_token(code, (pos1, pos2));
        if np2 - np1 > 0 {
            if let Some(tok) = check_keyword(&code[np1..np2]) {
                self.tokens.push_back((tok, self.cur_line, self.cur_column));
            } else if let Some(tok) = check_litteral(&code[np1..np2]) {
                self.tokens.push_back((tok, self.cur_line, self.cur_column));
            } else if let Some(tok) = check_namespace(&code[np1..np2]) {
                self.tokens.push_back((tok, self.cur_line, self.cur_column));
            }
            //At this point it has to be an identifier otherwise it's bad unexpected token
            else if let Some(tok) = check_identifier(&code[np1..np2]) {
                self.tokens.push_back((tok, self.cur_line, self.cur_column));
            } else {
                return Err(format!(
                    "[Shader Annotation Language] Unidentified token '{}' at line {}, column {}",
                    &code[np1..np2],
                    self.cur_line,
                    self.cur_column
                ));
            }
        }
        return Ok(());
    }

    pub fn push_str(&mut self, code: &str) -> Result<(), String>
    {
        self.cur_token = (0, 0);
        loop {
            let (mut pos1, mut pos2) = self.cur_token;
            self.cur_column += 1;
            pos2 += 1;
            if pos2 > code.len() {
                //Should be ">=" but somehow there's a bug in rust
                break;
            }
            if &code[pos2 - 1..pos2] == STR_COMMENT {
                self.in_comment = true;
            } else if &code[pos2 - 1..pos2] == "\r" || &code[pos2 - 1..pos2] == "\n" {
                if self.in_comment {
                    self.in_comment = false;
                    pos1 = pos2 + 1;
                    pos2 = pos1 + 1;
                }
                if &code[pos2 - 1..pos2] == "\n" {
                    self.cur_line += 1;
                    self.cur_column = 0;
                }
            }
            if !self.in_comment {
                if is_whitespace(&code[pos2 - 1..pos2]) {
                    self.parse_token(pos1, pos2, code)?;
                    pos1 = pos2; //This should be +1 but somehow there's a bug in rust
                    pos2 = pos1;
                }
            }
            self.cur_token = (pos1, pos2);
        }
        let (pos1, pos2) = self.cur_token;
        if pos2 + 1 < code.len() {
            //We have an error: input code is incomplete
            return Err(format!(
                "[Shader Annotation Language] Unexpected EOF at line {}, column {}",
                self.cur_line, self.cur_column
            ));
        }
        if pos2 - pos1 > 0 {
            self.parse_token(pos1, pos2, code)?;
        }
        return Ok(());
    }

    pub fn into_tokens(self) -> VecDeque<(Token, usize, usize)>
    {
        return self.tokens;
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    fn basic_assert(toks: Vec<Token>)
    {
        assert_eq!(
            toks,
            vec![
                Token::Const,
                Token::Identifier(String::from("float")),
                Token::Identifier(String::from("DeltaTime")),
                Token::Const,
                Token::Identifier(String::from("uint")),
                Token::Identifier(String::from("FrameCount")),
                Token::Const,
                Token::Identifier(String::from("mat3f")),
                Token::Identifier(String::from("ModelViewMatrix")),
                Token::Const,
                Token::Identifier(String::from("mat3f")),
                Token::Identifier(String::from("ProjectionMatrix")),
                Token::Const,
                Token::Struct,
                Token::Identifier(String::from("PerMaterial")),
                Token::BlockStart,
                Token::Identifier(String::from("vec4f")),
                Token::Identifier(String::from("BaseColor")),
                Token::Identifier(String::from("float")),
                Token::Identifier(String::from("UvMultiplier")),
                Token::BlockEnd
            ]
        );
    }

    #[test]
    fn basic_lexer()
    {
        let source_code = r"
            const float DeltaTime;
            const uint FrameCount;
            const mat3f ModelViewMatrix;
            const mat3f ProjectionMatrix;
            const struct PerMaterial
            {
                vec4f BaseColor;
                float UvMultiplier;
            }
        ";
        let mut lexer = Lexer::new();
        lexer.push_str(source_code).unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        basic_assert(toks);
    }

    #[test]
    fn lexer_comments()
    {
        let source_code = r"
            #this is a single line comment
            const float DeltaTime; # delta time
            const uint FrameCount; # frame count
            const mat3f ModelViewMatrix; # vew * model (2D only)
            const mat3f ProjectionMatrix; # projection (2D only)

            # Material
            const struct PerMaterial
            {
                vec4f BaseColor;
                float UvMultiplier;
            }
        ";
        let mut lexer = Lexer::new();
        lexer.push_str(source_code).unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        basic_assert(toks);
    }

    fn assert_typical(toks: Vec<Token>)
    {
        assert_eq!(
            toks,
            vec![
                Token::Vformat,
                Token::Struct,
                Token::Identifier(String::from("Vertex")),
                Token::BlockStart,
                Token::Identifier(String::from("vec4f")),
                Token::Identifier(String::from("Color")),
                Token::Identifier(String::from("vec3f")),
                Token::Identifier(String::from("Pos")),
                Token::Identifier(String::from("vec3f")),
                Token::Identifier(String::from("Normal")),
                Token::BlockEnd,
                Token::Const,
                Token::Struct,
                Token::Identifier(String::from("Projection")),
                Token::BlockStart,
                Token::Identifier(String::from("mat4f")),
                Token::Identifier(String::from("ProjectionMatrix")),
                Token::BlockEnd,
                Token::Const,
                Token::Identifier(String::from("mat4f")),
                Token::Identifier(String::from("ModelView"))
            ]
        );
    }

    #[test]
    fn typical_space()
    {
        let source_code = r"
            vformat struct Vertex
            {
                vec4f Color;
                vec3f Pos;
                vec3f Normal;
            }

            const struct Projection
            {
                mat4f ProjectionMatrix;
            }

            const mat4f ModelView;
        ";
        let mut lexer = Lexer::new();
        lexer.push_str(source_code).unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        assert_typical(toks);
    }

    #[test]
    fn typical_no_space()
    {
        let source_code = r"
vformat struct Vertex
{
    vec4f Color;
    vec3f Pos;
    vec3f Normal;
}

const struct Projection
{
    mat4f ProjectionMatrix;
}

const mat4f ModelView;
";
        let mut lexer = Lexer::new();
        lexer.push_str(source_code).unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        assert_typical(toks);
    }

    #[test]
    fn typical_line_per_line()
    {
        let mut lexer = Lexer::new();
        lexer.push_str("vformat struct Vertex").unwrap();
        lexer.push_str("{").unwrap();
        lexer.push_str("    vec4f Color;").unwrap();
        lexer.push_str("    vec3f Pos;").unwrap();
        lexer.push_str("    vec3f Normal;").unwrap();
        lexer.push_str("}").unwrap();
        lexer.push_str("").unwrap();
        lexer.push_str("const struct Projection").unwrap();
        lexer.push_str("{").unwrap();
        lexer.push_str("    mat4f ProjectionMatrix;").unwrap();
        lexer.push_str("}").unwrap();
        lexer.push_str("").unwrap();
        lexer.push_str("const mat4f ModelView;").unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        assert_typical(toks);
    }

    #[test]
    fn lexer_non_trailing()
    {
        let source_code = r"output vec4f color;";
        let mut lexer = Lexer::new();
        lexer.push_str(source_code).unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        assert_eq!(
            toks,
            vec![
                Token::Output,
                Token::Identifier(String::from("vec4f")),
                Token::Identifier(String::from("color"))
            ]
        );
    }

    #[test]
    fn lexer_whitespace1()
    {
        let source_code = r"  output	  	vec4f  	  color;	 ";
        let mut lexer = Lexer::new();
        lexer.push_str(source_code).unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        assert_eq!(
            toks,
            vec![
                Token::Output,
                Token::Identifier(String::from("vec4f")),
                Token::Identifier(String::from("color"))
            ]
        );
    }

    #[test]
    fn lexer_whitespace2()
    {
        let source_code = r"output	  	vec4f  	  color;";
        let mut lexer = Lexer::new();
        lexer.push_str(source_code).unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        assert_eq!(
            toks,
            vec![
                Token::Output,
                Token::Identifier(String::from("vec4f")),
                Token::Identifier(String::from("color"))
            ]
        );
    }

    #[test]
    fn lexer_use()
    {
        let source_code = r"use test::test;";
        let mut lexer = Lexer::new();
        lexer.push_str(source_code).unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        assert_eq!(toks, vec![Token::Use, Token::Namespace(String::from("test::test"))]);
    }

    #[test]
    fn lexer_outputs()
    {
        let source_code = r"#this is a single line comment
            output vec4f color;
        ";
        let mut lexer = Lexer::new();
        lexer.push_str(source_code).unwrap();
        let toks: Vec<Token> = lexer.into_tokens().iter().map(|(v, _, __)| v.clone()).collect();
        assert_eq!(
            toks,
            vec![
                Token::Output,
                Token::Identifier(String::from("vec4f")),
                Token::Identifier(String::from("color"))
            ]
        );
    }
}

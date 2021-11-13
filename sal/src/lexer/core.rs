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

use std::{collections::VecDeque, str::from_utf8_unchecked};

use regex::bytes::Regex;

use crate::lexer::{
    error::Error,
    token::{
        Token,
        CHR_BLOCK_END,
        CHR_BLOCK_START,
        CHR_BREAK,
        CHR_COLON,
        CHR_COMMENT,
        CHR_EQ,
        CHR_NL,
        STR_BLENDFUNC,
        STR_CONST,
        STR_FALSE,
        STR_OUTPUT,
        STR_PIPELINE,
        STR_STRUCT,
        STR_TRUE,
        STR_USE,
        STR_VFORMAT
    }
};

pub struct TokenEntry
{
    pub line: usize,
    pub col: usize,
    pub token: Token
}

fn check_punct(chr: u8) -> Option<Token>
{
    match chr {
        CHR_EQ => Some(Token::Eq),
        CHR_BLOCK_START => Some(Token::BlockStart),
        CHR_BLOCK_END => Some(Token::BlockEnd),
        _ => None
    }
}

fn check_keyword(substr: &[u8]) -> Option<Token>
{
    if substr.len() == 1 {
        check_punct(substr[0])
    } else {
        match substr {
            STR_CONST => Some(Token::Const),
            STR_STRUCT => Some(Token::Struct),
            STR_PIPELINE => Some(Token::Pipeline),
            STR_VFORMAT => Some(Token::Vformat),
            STR_BLENDFUNC => Some(Token::Blendfunc),
            STR_USE => Some(Token::Use),
            STR_OUTPUT => Some(Token::Output),
            _ => None
        }
    }
}

fn check_litteral(substr: &[u8]) -> Option<Token>
{
    if substr == STR_TRUE {
        return Some(Token::Bool(true));
    } else if substr == STR_FALSE {
        return Some(Token::Bool(false));
    }
    //^\d+$
    let int = Regex::new(r"^\d+$").unwrap();
    let float = Regex::new(r"^\d*.\d+$").unwrap();
    if int.is_match(substr) {
        //SAFETY: If we get there and that we don't have a valid int well then regex crate is broken!
        unsafe {
            return Some(Token::Int(from_utf8_unchecked(substr).parse().unwrap()));
        }
    }
    if float.is_match(substr) {
        //SAFETY: If we get there and that we don't have a valid float well then regex crate is broken!
        unsafe {
            return Some(Token::Float(from_utf8_unchecked(substr).parse().unwrap()));
        }
    }
    None
}

fn check_identifier(substr: &[u8]) -> Option<Token>
{
    let re = Regex::new(r"^[A-z]([A-z]|\d)*$").unwrap();
    if re.is_match(substr) {
        //SAFETY: If we get there but substr is not valid UTF8 well then regex crate is broken!
        unsafe {
            return Some(Token::Identifier(from_utf8_unchecked(substr).into()));
        }
    }
    None
}

fn check_terminator(chr: u8) -> Option<Token>
{
    if is_whitespace(chr) {
        Some(Token::Whitespace)
    } else {
        match chr {
            CHR_BREAK => Some(Token::Break),
            CHR_COLON => Some(Token::Colon),
            _ => None
        }
    }
}

fn is_whitespace(chr: u8) -> bool
{
    matches!(chr, b'\t' | b' ' | b'\r' | CHR_NL)
}

fn trim_token(code: &[u8], token: (usize, usize)) -> (usize, usize)
{
    let (mut pos1, mut pos2) = token;
    while pos1 < pos2 && is_whitespace(code[pos1]) {
        pos1 += 1;
    }
    while pos2 > pos1 && is_whitespace(code[pos2 - 1]) {
        pos2 -= 1;
    }
    (pos1, pos2)
}

pub struct Lexer
{
    tokens: VecDeque<TokenEntry>,
    cur_token: (usize, usize),
    cur_line: usize,
    cur_column: usize,
    in_comment: bool
}

impl Default for Lexer
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Lexer
{
    pub fn new() -> Lexer
    {
        Lexer {
            tokens: VecDeque::new(),
            cur_token: (0, 0),
            cur_column: 0,
            cur_line: 1,
            in_comment: false
        }
    }

    fn parse_token(&mut self, pos1: usize, pos2: usize, code: &[u8]) -> Result<(), Error>
    {
        let (np1, np2) = trim_token(code, (pos1, pos2));
        if np2 - np1 > 0 {
            if let Some(tok) = check_keyword(&code[np1..np2]) {
                self.tokens.push_back(TokenEntry {
                    token: tok,
                    line: self.cur_line,
                    col: self.cur_column
                });
            } else if let Some(tok) = check_litteral(&code[np1..np2]) {
                self.tokens.push_back(TokenEntry {
                    token: tok,
                    line: self.cur_line,
                    col: self.cur_column
                });
            }
            //At this point it has to be an identifier otherwise it's a bad unexpected token
            else if let Some(tok) = check_identifier(&code[np1..np2]) {
                self.tokens.push_back(TokenEntry {
                    token: tok,
                    line: self.cur_line,
                    col: self.cur_column
                });
            } else {
                return Err(Error::unidentified_token(
                    self.cur_line,
                    self.cur_column,
                    &code[np1..np2]
                ));
            }
        }
        Ok(())
    }

    pub fn process(&mut self, code: &[u8]) -> Result<(), Error>
    {
        self.cur_token = (0, 0);
        loop {
            let (mut pos1, mut pos2) = self.cur_token;
            self.cur_column += 1;
            pos2 += 1;
            if pos2 > code.len() {
                break;
            }
            if code[pos2 - 1] == CHR_COMMENT {
                self.in_comment = true;
            } else if code[pos2 - 1] == CHR_NL {
                if self.in_comment {
                    self.in_comment = false;
                    pos1 = pos2 + 1;
                    pos2 = pos1 + 1;
                }
                if code[pos2 - 1] == b'\n' {
                    self.cur_line += 1;
                    self.cur_column = 0;
                }
            }
            if !self.in_comment {
                if let Some(tok) = check_terminator(code[pos2 - 1]) {
                    self.parse_token(pos1, pos2 - 1, code)?;
                    pos1 = pos2; //This should be +1 but somehow there's a strange thing here
                    self.tokens.push_back(TokenEntry {
                        token: tok,
                        line: self.cur_line,
                        col: self.cur_column
                    });
                }
            }
            self.cur_token = (pos1, pos2);
        }
        let (pos1, pos2) = self.cur_token;
        if pos2 + 1 < code.len() {
            //We have an error: input code is incomplete
            return Err(Error::eof(self.cur_line, self.cur_column));
        }
        if pos2 - pos1 > 0 {
            self.parse_token(pos1, pos2, code)?;
        }
        Ok(())
    }

    pub fn eliminate_whitespace(&mut self)
    {
        self.tokens
            .retain(|TokenEntry { token, .. }| token != &Token::Whitespace);
    }

    pub fn eliminate_breaks(&mut self)
    {
        self.tokens.retain(|TokenEntry { token, .. }| token != &Token::Break);
    }

    pub fn into_tokens(self) -> VecDeque<TokenEntry>
    {
        self.tokens
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
                Token::Identifier("float".into()),
                Token::Identifier("DeltaTime".into()),
                Token::Const,
                Token::Identifier("uint".into()),
                Token::Identifier("FrameCount".into()),
                Token::Const,
                Token::Identifier("mat3f".into()),
                Token::Identifier("ModelViewMatrix".into()),
                Token::Const,
                Token::Identifier("mat3f".into()),
                Token::Identifier("ProjectionMatrix".into()),
                Token::Const,
                Token::Struct,
                Token::Identifier("PerMaterial".into()),
                Token::BlockStart,
                Token::Identifier("vec4f".into()),
                Token::Identifier("BaseColor".into()),
                Token::Identifier("float".into()),
                Token::Identifier("UvMultiplier".into()),
                Token::BlockEnd
            ]
        );
    }

    #[test]
    fn basic_lexer()
    {
        let source_code = b"
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
        lexer.process(source_code).unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        basic_assert(toks);
    }

    #[test]
    fn lexer_comments()
    {
        let source_code = b"
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
        lexer.process(source_code).unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        basic_assert(toks);
    }

    fn assert_typical(toks: Vec<Token>)
    {
        assert_eq!(
            toks,
            vec![
                Token::Vformat,
                Token::Struct,
                Token::Identifier("Vertex".into()),
                Token::BlockStart,
                Token::Identifier("vec4f".into()),
                Token::Identifier("Color".into()),
                Token::Identifier("vec3f".into()),
                Token::Identifier("Pos".into()),
                Token::Identifier("vec3f".into()),
                Token::Identifier("Normal".into()),
                Token::BlockEnd,
                Token::Const,
                Token::Struct,
                Token::Identifier("Projection".into()),
                Token::BlockStart,
                Token::Identifier("mat4f".into()),
                Token::Identifier("ProjectionMatrix".into()),
                Token::BlockEnd,
                Token::Const,
                Token::Identifier("mat4f".into()),
                Token::Identifier("ModelView".into())
            ]
        );
    }

    #[test]
    fn typical_space()
    {
        let source_code = b"
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
        lexer.process(source_code).unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        assert_typical(toks);
    }

    #[test]
    fn typical_no_space()
    {
        let source_code = b"
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
        lexer.process(source_code).unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        assert_typical(toks);
    }

    #[test]
    fn typical_line_per_line()
    {
        let mut lexer = Lexer::new();
        lexer.process(b"vformat struct Vertex").unwrap();
        lexer.process(b"{").unwrap();
        lexer.process(b"    vec4f Color;").unwrap();
        lexer.process(b"    vec3f Pos;").unwrap();
        lexer.process(b"    vec3f Normal;").unwrap();
        lexer.process(b"}").unwrap();
        lexer.process(b"").unwrap();
        lexer.process(b"const struct Projection").unwrap();
        lexer.process(b"{").unwrap();
        lexer.process(b"    mat4f ProjectionMatrix;").unwrap();
        lexer.process(b"}").unwrap();
        lexer.process(b"").unwrap();
        lexer.process(b"const mat4f ModelView;").unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        assert_typical(toks);
    }

    #[test]
    fn lexer_non_trailing()
    {
        let source_code = b"output vec4f color;";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        assert_eq!(
            toks,
            vec![
                Token::Output,
                Token::Identifier("vec4f".into()),
                Token::Identifier("color".into())
            ]
        );
    }

    #[test]
    fn lexer_whitespace1()
    {
        let source_code = b"  output	  	vec4f  	  color;	 ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        assert_eq!(
            toks,
            vec![
                Token::Output,
                Token::Identifier("vec4f".into()),
                Token::Identifier("color".into())
            ]
        );
    }

    #[test]
    fn lexer_whitespace2()
    {
        let source_code = b"output	  	vec4f  	  color;";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        assert_eq!(
            toks,
            vec![
                Token::Output,
                Token::Identifier("vec4f".into()),
                Token::Identifier("color".into())
            ]
        );
    }

    #[test]
    fn lexer_use()
    {
        let source_code = b"use test::test;";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        assert_eq!(
            toks,
            vec![
                Token::Use,
                Token::Identifier("test".into()),
                Token::Colon,
                Token::Colon,
                Token::Identifier("test".into())
            ]
        );
    }

    #[test]
    fn lexer_outputs()
    {
        let source_code = b"#this is a single line comment
            output vec4f color;
        ";
        let mut lexer = Lexer::new();
        lexer.process(source_code).unwrap();
        lexer.eliminate_whitespace();
        lexer.eliminate_breaks();
        let toks: Vec<Token> = lexer
            .into_tokens()
            .iter()
            .map(|TokenEntry { token, .. }| token.clone())
            .collect();
        assert_eq!(
            toks,
            vec![
                Token::Output,
                Token::Identifier("vec4f".into()),
                Token::Identifier("color".into())
            ]
        );
    }
}

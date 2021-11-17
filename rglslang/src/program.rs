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

use std::{borrow::Cow, ffi::CStr};

use glslang_sys::{
    EShMessages,
    EShMsgDefault,
    EShReflectionAllBlockVariables,
    EShReflectionAllIOVariables,
    EShReflectionBasicArraySuffix,
    EShReflectionDefault,
    EShReflectionIntermediateIO,
    EShReflectionOptions,
    EShReflectionSeparateBuffers,
    EShReflectionSharedStd140SSBO,
    EShReflectionSharedStd140UBO,
    EShReflectionStrictArraySuffix,
    EShReflectionUnwrapIOBlocks,
    TProgram,
    TProgram_addShader,
    TProgram_buildReflection,
    TProgram_create,
    TProgram_destroy,
    TProgram_getInfoDebugLog,
    TProgram_getInfoLog,
    TProgram_link,
    TShader,
    TShader_destroy
};

use crate::shader::{unwrap_messages, unwrap_shader, Messages, Shader};

pub struct ReflectionOptions
{
    opts: EShReflectionOptions
}

impl ReflectionOptions
{
    pub fn new() -> Self
    {
        Self {
            opts: EShReflectionDefault
        }
    }

    pub fn strict_array_suffix(mut self) -> Self
    {
        self.opts |= EShReflectionStrictArraySuffix;
        self
    }

    pub fn basic_array_suffix(mut self) -> Self
    {
        self.opts |= EShReflectionBasicArraySuffix;
        self
    }

    pub fn intermediate_io(mut self) -> Self
    {
        self.opts |= EShReflectionIntermediateIO;
        self
    }

    pub fn seperate_buffers(mut self) -> Self
    {
        self.opts |= EShReflectionSeparateBuffers;
        self
    }

    pub fn all_block_variables(mut self) -> Self
    {
        self.opts |= EShReflectionAllBlockVariables;
        self
    }

    pub fn unwrap_io_blocks(mut self) -> Self
    {
        self.opts |= EShReflectionUnwrapIOBlocks;
        self
    }

    pub fn all_io_variables(mut self) -> Self
    {
        self.opts |= EShReflectionAllIOVariables;
        self
    }

    pub fn shared_std140_ssbo(mut self) -> Self
    {
        self.opts |= EShReflectionSharedStd140SSBO;
        self
    }

    pub fn shared_std140_ubo(mut self) -> Self
    {
        self.opts |= EShReflectionSharedStd140UBO;
        self
    }
}

pub struct Builder
{
    shaders: Vec<*const TShader>,
    low_level: *const TProgram,
    reflection: Option<EShReflectionOptions>,
    messages: EShMessages
}

impl Builder
{
    pub fn new() -> Self
    {
        unsafe {
            Self {
                shaders: Vec::new(),
                low_level: TProgram_create(),
                reflection: None,
                messages: EShMsgDefault
            }
        }
    }

    pub fn add_shader(mut self, shader: Shader) -> Self
    {
        let ptr = unwrap_shader(shader);
        self.shaders.push(ptr);
        unsafe {
            TProgram_addShader(self.low_level, ptr);
        }
        self
    }

    pub fn enable_reflection(mut self, options: ReflectionOptions) -> Self
    {
        self.reflection = Some(options.opts);
        self
    }

    pub fn messages(mut self, msgs: Messages) -> Self
    {
        self.messages = unwrap_messages(msgs);
        self
    }

    pub fn link(self) -> Program
    {
        unsafe {
            let mut flag = true;
            if let Some(opts) = self.reflection {
                flag = TProgram_buildReflection(self.low_level, opts);
            }
            if flag {
                flag = TProgram_link(self.low_level, self.messages);
            }
            Program {
                shaders: self.shaders,
                low_level: self.low_level,
                valid: flag
            }
        }
    }
}

pub struct Program
{
    shaders: Vec<*const TShader>,
    low_level: *const TProgram,
    valid: bool
}

impl Program
{
    pub fn get_info_log(&self) -> Cow<str>
    {
        unsafe {
            let str = CStr::from_ptr(TProgram_getInfoLog(self.low_level));
            str.to_string_lossy()
        }
    }

    pub fn get_info_debug_log(&self) -> Cow<str>
    {
        unsafe {
            let str = CStr::from_ptr(TProgram_getInfoDebugLog(self.low_level));
            str.to_string_lossy()
        }
    }

    pub fn check(&self) -> bool
    {
        self.valid
    }
}

impl Drop for Program
{
    fn drop(&mut self)
    {
        unsafe {
            TProgram_destroy(self.low_level);
            for s in &self.shaders {
                TShader_destroy(*s);
            }
        }
    }
}

// TODO: Make sure this is REALLY safe
// SAFETY: This is a wild guess considering the use of locks for the globals in the source code of glslang
unsafe impl Send for Program {}

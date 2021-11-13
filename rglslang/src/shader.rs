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
    ffi::{CStr, CString},
    os::raw::c_char
};

use glslang_sys::{
    limits::TBuiltInResource_default,
    versions::{ECompatibilityProfile, ECoreProfile, EEsProfile, ENoProfile, EProfile},
    EShMessages,
    EShMsgAST,
    EShMsgDebugInfo,
    EShMsgDefault,
    EShMsgRelaxedErrors,
    EShMsgSuppressWarnings,
    EShSourceGlsl,
    EShTexSampTransUpgradeTextureRemoveSampler,
    EbsCount,
    EbsNone,
    EbsPushConstant,
    EbsStorageBuffer,
    EbsUniform,
    TBlockStorageClass,
    TShader,
    TShader_addBlockStorageOverride,
    TShader_addUniformLocationOverride,
    TShader_create,
    TShader_destroy,
    TShader_getInfoDebugLog,
    TShader_getInfoLog,
    TShader_parse,
    TShader_setAtomicCounterBlockBinding,
    TShader_setAtomicCounterBlockName,
    TShader_setAtomicCounterBlockSet,
    TShader_setAutoMapBindings,
    TShader_setAutoMapLocations,
    TShader_setEntryPoint,
    TShader_setEnvClient,
    TShader_setEnvInput,
    TShader_setEnvInputVulkanRulesRelaxed,
    TShader_setEnvTarget,
    TShader_setGlobalUniformBinding,
    TShader_setGlobalUniformBlockName,
    TShader_setGlobalUniformSet,
    TShader_setInvertY,
    TShader_setNanMinMaxClamp,
    TShader_setNoStorageFormat,
    TShader_setPreamble,
    TShader_setSourceEntryPoint,
    TShader_setStringsWithLengthsAndNames,
    TShader_setTextureSamplerTransformMode,
    TShader_setUniformLocationBase,
    TShader_setUniqueId
};

use crate::environment::Environment;

#[derive(Copy, Clone, Debug)]
pub enum Profile
{
    None,
    Core,
    Compatibility,
    Es
}

impl Profile
{
    pub fn into(self) -> EProfile
    {
        return match self {
            Profile::None => ENoProfile,
            Profile::Core => ECoreProfile,
            Profile::Compatibility => ECompatibilityProfile,
            Profile::Es => EEsProfile
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BlockStorageClass
{
    Uniform,
    StorageBuffer,
    PushConstant,
    None,
    Count
}

impl BlockStorageClass
{
    pub fn into(self) -> TBlockStorageClass
    {
        return match self {
            BlockStorageClass::Uniform => EbsUniform,
            BlockStorageClass::StorageBuffer => EbsStorageBuffer,
            BlockStorageClass::PushConstant => EbsPushConstant,
            BlockStorageClass::None => EbsNone,
            BlockStorageClass::Count => EbsCount
        };
    }
}

#[derive(Clone, Debug)]
pub struct Part
{
    code: String,          //Source code
    name: Option<CString>  //Optional name of source code
}

impl Part
{
    pub fn new<T: AsRef<str>>(code: T) -> Part
    {
        return Part {
            code: String::from(code.as_ref()),
            name: None
        };
    }

    pub fn new_with_name<T: AsRef<str>, T1: AsRef<str>>(code: T, name: T1) -> Part
    {
        return Part {
            code: String::from(code.as_ref()),
            name: Some(CString::new(name.as_ref()).unwrap())
        };
    }
}

#[derive(Default)]
struct ShaderStorage
{
    parts: Vec<Part>,
    code_len_arr: Vec<i32>,
    code_arr: Vec<*const c_char>,
    name_arr: Vec<*const c_char>,
    preamble: Option<CString>,
    entry_point: Option<CString>,
    source_entry_point_name: Option<CString>,
    global_uniform_block_name: Option<CString>,
    atomic_counter_block_name: Option<CString>,
    uniform_location_overrides: Vec<CString>,
    block_storage_overrides: Vec<CString>
}

impl ShaderStorage
{
    fn build_code_name_arr(&mut self)
    {
        for v in &self.parts {
            self.code_len_arr.push(v.code.len() as _);
            self.code_arr.push(v.code.as_ptr() as _);
            if let Some(n) = &v.name {
                self.name_arr.push(n.as_ptr());
            } else {
                self.name_arr.push(std::ptr::null());
            }
        }
    }
}

pub struct Messages
{
    messages: EShMessages
}

impl Messages
{
    pub fn new() -> Messages
    {
        return Messages {
            messages: EShMsgDefault
        };
    }

    pub fn suppress_warnings(mut self) -> Self
    {
        self.messages |= EShMsgSuppressWarnings;
        return self;
    }

    pub fn debug(mut self) -> Self
    {
        self.messages |= EShMsgDebugInfo;
        return self;
    }

    pub fn relaxed_errors(mut self) -> Self
    {
        self.messages |= EShMsgRelaxedErrors;
        return self;
    }

    pub fn ast(mut self) -> Self
    {
        self.messages |= EShMsgAST;
        return self;
    }
}

pub struct Builder
{
    storage: ShaderStorage,
    low_level: *const TShader,
    env: Environment,
    default_version: i32,
    default_profile: Profile,
    forward_compatible: bool,
    force_default_version_and_profile: bool,
    messages: EShMessages
}

impl Builder
{
    pub fn new(env: Environment) -> Builder
    {
        unsafe {
            return Builder {
                storage: ShaderStorage::default(),
                low_level: TShader_create(env.get_stage()),
                env,
                default_profile: Profile::None,
                default_version: 300,
                forward_compatible: true,
                force_default_version_and_profile: false,
                messages: EShMsgDefault
            };
        }
    }

    pub fn preamble<T: AsRef<str>>(mut self, preamble: T) -> Self
    {
        self.storage.preamble = Some(CString::new(preamble.as_ref()).unwrap());
        unsafe {
            TShader_setPreamble(self.low_level, self.storage.preamble.as_ref().unwrap().as_ptr());
        }
        return self;
    }

    pub fn entry_point<T: AsRef<str>>(mut self, name: T) -> Self
    {
        self.storage.entry_point = Some(CString::new(name.as_ref()).unwrap());
        unsafe {
            TShader_setEntryPoint(self.low_level, self.storage.entry_point.as_ref().unwrap().as_ptr());
        }
        return self;
    }

    pub fn source_entry_point<T: AsRef<str>>(mut self, name: T) -> Self
    {
        self.storage.source_entry_point_name = Some(CString::new(name.as_ref()).unwrap());
        unsafe {
            TShader_setSourceEntryPoint(
                self.low_level,
                self.storage.source_entry_point_name.as_ref().unwrap().as_ptr()
            );
        }
        return self;
    }

    pub fn global_uniform_block_name<T: AsRef<str>>(mut self, name: T) -> Self
    {
        self.storage.global_uniform_block_name = Some(CString::new(name.as_ref()).unwrap());
        unsafe {
            TShader_setGlobalUniformBlockName(
                self.low_level,
                self.storage.global_uniform_block_name.as_ref().unwrap().as_ptr()
            );
        }
        return self;
    }

    pub fn atomic_counter_block_name<T: AsRef<str>>(mut self, name: T) -> Self
    {
        self.storage.atomic_counter_block_name = Some(CString::new(name.as_ref()).unwrap());
        unsafe {
            TShader_setAtomicCounterBlockName(
                self.low_level,
                self.storage.atomic_counter_block_name.as_ref().unwrap().as_ptr()
            );
        }
        return self;
    }

    pub fn add_block_storage_override<T: AsRef<str>>(mut self, name: T, backing: BlockStorageClass) -> Self
    {
        self.storage
            .block_storage_overrides
            .push(CString::new(name.as_ref()).unwrap());
        let name = self.storage.block_storage_overrides.last().unwrap();
        unsafe {
            TShader_addBlockStorageOverride(self.low_level, name.as_ptr(), backing.into());
        }
        return self;
    }

    pub fn add_uniform_location_override<T: AsRef<str>>(mut self, name: T, loc: i32) -> Self
    {
        self.storage
            .uniform_location_overrides
            .push(CString::new(name.as_ref()).unwrap());
        let name = self.storage.uniform_location_overrides.last().unwrap();
        unsafe {
            TShader_addUniformLocationOverride(self.low_level, name.as_ptr(), loc);
        }
        return self;
    }

    pub fn add_part(mut self, p: Part) -> Self
    {
        self.storage.parts.push(p);
        return self;
    }

    pub fn unique_id(self, id: u64) -> Self
    {
        unsafe {
            TShader_setUniqueId(self.low_level, id);
        }
        return self;
    }

    pub fn auto_map_bindings(self) -> Self
    {
        unsafe {
            TShader_setAutoMapBindings(self.low_level, true);
        }
        return self;
    }

    pub fn auto_map_locations(self) -> Self
    {
        unsafe {
            TShader_setAutoMapLocations(self.low_level, true);
        }
        return self;
    }

    pub fn uniform_location_base(self, base: i32) -> Self
    {
        unsafe {
            TShader_setUniformLocationBase(self.low_level, base);
        }
        return self;
    }

    pub fn invert_y(self) -> Self
    {
        unsafe {
            TShader_setInvertY(self.low_level, true);
        }
        return self;
    }

    pub fn no_storage_format(self) -> Self
    {
        unsafe {
            TShader_setNoStorageFormat(self.low_level, true);
        }
        return self;
    }

    pub fn nan_min_max_clamp(self) -> Self
    {
        unsafe {
            TShader_setNanMinMaxClamp(self.low_level, true);
        }
        return self;
    }

    pub fn use_combined_texture_sampler(self) -> Self
    {
        unsafe {
            TShader_setTextureSamplerTransformMode(self.low_level, EShTexSampTransUpgradeTextureRemoveSampler);
        }
        return self;
    }

    pub fn global_uniform_set(self, set: u32) -> Self
    {
        unsafe {
            TShader_setGlobalUniformSet(self.low_level, set);
        }
        return self;
    }

    pub fn global_uniform_binding(self, binding: u32) -> Self
    {
        unsafe {
            TShader_setGlobalUniformBinding(self.low_level, binding);
        }
        return self;
    }

    pub fn atomic_counter_block_set(self, set: u32) -> Self
    {
        unsafe {
            TShader_setAtomicCounterBlockSet(self.low_level, set);
        }
        return self;
    }

    pub fn atomic_counter_block_binding(self, binding: u32) -> Self
    {
        unsafe {
            TShader_setAtomicCounterBlockBinding(self.low_level, binding);
        }
        return self;
    }

    pub fn vulkan_rules_relaxed(self) -> Self
    {
        unsafe {
            TShader_setEnvInputVulkanRulesRelaxed(self.low_level);
        }
        return self;
    }

    pub fn default_version(mut self, version: i32) -> Self
    {
        self.default_version = version;
        return self;
    }

    pub fn default_profile(mut self, profile: Profile) -> Self
    {
        self.default_profile = profile;
        return self;
    }

    pub fn forward_incompatible(mut self) -> Self
    {
        self.forward_compatible = false;
        return self;
    }

    pub fn force_default_version_and_profile(mut self) -> Self
    {
        self.force_default_version_and_profile = true;
        return self;
    }

    pub fn messages(mut self, msgs: Messages) -> Self
    {
        self.messages = msgs.messages;
        return self;
    }

    pub fn parse(mut self) -> Shader
    {
        unsafe {
            TShader_setEnvInput(
                self.low_level,
                EShSourceGlsl,
                self.env.get_stage(),
                self.env.get_dialect(),
                self.env.get_dialect_version()
            );
            TShader_setEnvClient(self.low_level, self.env.get_client(), self.env.get_client_version());
            TShader_setEnvTarget(
                self.low_level,
                self.env.get_target_language(),
                self.env.get_target_language_version()
            );
            self.storage.build_code_name_arr();
            TShader_setStringsWithLengthsAndNames(
                self.low_level,
                self.storage.code_arr.as_ptr(),
                self.storage.code_len_arr.as_ptr(),
                self.storage.name_arr.as_ptr(),
                self.storage.code_arr.len() as _
            );
            let flag = TShader_parse(
                self.low_level,
                TBuiltInResource_default(),
                self.default_version,
                self.default_profile.into(),
                self.force_default_version_and_profile,
                self.forward_compatible,
                self.messages
            );
            return Shader {
                valid: flag,
                _storage: self.storage,
                low_level: self.low_level
            };
        }
    }
}

pub struct Shader
{
    valid: bool,
    _storage: ShaderStorage,
    low_level: *const TShader
}

impl Shader
{
    pub fn get_info_log(&self) -> &str
    {
        unsafe {
            let str = CStr::from_ptr(TShader_getInfoLog(self.low_level));
            return str.to_str().unwrap();
        }
    }

    pub fn get_info_debug_log(&self) -> &str
    {
        unsafe {
            let str = CStr::from_ptr(TShader_getInfoDebugLog(self.low_level));
            return str.to_str().unwrap();
        }
    }

    pub fn check(&self) -> bool
    {
        return self.valid;
    }
}

impl Drop for Shader
{
    fn drop(&mut self)
    {
        unsafe {
            TShader_destroy(self.low_level);
        }
    }
}

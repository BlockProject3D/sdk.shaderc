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

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

pub mod limits;
pub mod versions;

use std::os::raw::{c_char, c_int, c_uint, c_ulonglong, c_void};

/// typedef enum EShLanguage
#[repr(transparent)]
pub struct EShLanguage(c_int);

pub const EShLangVertex: EShLanguage = EShLanguage(0);
pub const EShLangTessControl: EShLanguage = EShLanguage(1);
pub const EShLangTessEvaluation: EShLanguage = EShLanguage(2);
pub const EShLangGeometry: EShLanguage = EShLanguage(3);
pub const EShLangFragment: EShLanguage = EShLanguage(4);
pub const EShLangCompute: EShLanguage = EShLanguage(5);
pub const EShLangRayGen: EShLanguage = EShLanguage(6);
pub const EShLangRayGenNV: EShLanguage = EShLangRayGen;
pub const EShLangIntersect: EShLanguage = EShLanguage(7);
pub const EShLangIntersectNV: EShLanguage = EShLangIntersect;
pub const EShLangAnyHit: EShLanguage = EShLanguage(8);
pub const EShLangAnyHitNV: EShLanguage = EShLangAnyHit;
pub const EShLangClosestHit: EShLanguage = EShLanguage(9);
pub const EShLangClosestHitNV: EShLanguage = EShLangClosestHit;
pub const EShLangMiss: EShLanguage = EShLanguage(10);
pub const EShLangMissNV: EShLanguage = EShLangMiss;
pub const EShLangCallable: EShLanguage = EShLanguage(11);
pub const EShLangCallableNV: EShLanguage = EShLangCallable;
pub const EShLangTaskNV: EShLanguage = EShLanguage(12);
pub const EShLangMeshNV: EShLanguage = EShLanguage(13);

/// typedef enum EShLanguageMask (bitflags)
pub type EShLanguageMask = c_uint;

pub const EShLangVertexMask: EShLanguageMask = 1 << EShLangVertex.0;
pub const EShLangTessControlMask: EShLanguageMask = 1 << EShLangTessControl.0;
pub const EShLangTessEvaluationMask: EShLanguageMask = 1 << EShLangTessEvaluation.0;
pub const EShLangGeometryMask: EShLanguageMask = 1 << EShLangGeometry.0;
pub const EShLangFragmentMask: EShLanguageMask = 1 << EShLangFragment.0;
pub const EShLangComputeMask: EShLanguageMask = 1 << EShLangCompute.0;
pub const EShLangRayGenMask: EShLanguageMask = 1 << EShLangRayGen.0;
pub const EShLangRayGenNVMask: EShLanguageMask = EShLangRayGenMask;
pub const EShLangIntersectMask: EShLanguageMask = 1 << EShLangIntersect.0;
pub const EShLangIntersectNVMask: EShLanguageMask = EShLangIntersectMask;
pub const EShLangAnyHitMask: EShLanguageMask = 1 << EShLangAnyHit.0;
pub const EShLangAnyHitNVMask: EShLanguageMask = EShLangAnyHitMask;
pub const EShLangClosestHitMask: EShLanguageMask = 1 << EShLangClosestHit.0;
pub const EShLangClosestHitNVMask: EShLanguageMask = EShLangClosestHitMask;
pub const EShLangMissMask: EShLanguageMask = 1 << EShLangMiss.0;
pub const EShLangMissNVMask: EShLanguageMask = EShLangMissMask;
pub const EShLangCallableMask: EShLanguageMask = 1 << EShLangCallable.0;
pub const EShLangCallableNVMask: EShLanguageMask = EShLangCallableMask;
pub const EShLangTaskNVMask: EShLanguageMask = 1 << EShLangTaskNV.0;
pub const EShLangMeshNVMask: EShLanguageMask = 1 << EShLangMeshNV.0;

/// typedef enum EShSource
#[repr(transparent)]
pub struct EShSource(c_int);

pub const EShSourceNone: EShSource = EShSource(0);
pub const EShSourceGlsl: EShSource = EShSource(1);
pub const EShSourceHlsl: EShSource = EShSource(2);

/// typedef enum EShClient
#[repr(transparent)]
pub struct EShClient(c_int);

pub const EShClientNone: EShClient = EShClient(0);
pub const EShClientVulkan: EShClient = EShClient(1);
pub const EShClientOpenGL: EShClient = EShClient(2);

/// typedef enum EShTargetLanguage
#[repr(transparent)]
pub struct EShTargetLanguage(c_int);

pub const EShTargetNone: EShTargetLanguage = EShTargetLanguage(0);
pub const EShTargetSpv: EShTargetLanguage = EShTargetLanguage(1);

/// typedef enum EShTargetClientVersion
#[repr(transparent)]
pub struct EShTargetClientVersion(c_int);

pub const EShTargetVersionNone: EShTargetClientVersion = EShTargetClientVersion(0);
pub const EShTargetVulkan_1_0: EShTargetClientVersion = EShTargetClientVersion(1 << 22);
pub const EShTargetVulkan_1_1: EShTargetClientVersion = EShTargetClientVersion((1 << 22) | (1 << 12));
pub const EShTargetVulkan_1_2: EShTargetClientVersion = EShTargetClientVersion((1 << 22) | (2 << 12));
pub const EShTargetOpenGL_450: EShTargetClientVersion = EShTargetClientVersion(450);

/// typedef enum EShTargetLanguageVersion
#[repr(transparent)]
pub struct EShTargetLanguageVersion(c_int);

pub const EShTargetLangNone: EShTargetLanguageVersion = EShTargetLanguageVersion(0);
pub const EShTargetSpv_1_0: EShTargetLanguageVersion = EShTargetLanguageVersion(1 << 16);
pub const EShTargetSpv_1_1: EShTargetLanguageVersion = EShTargetLanguageVersion((1 << 16) | (1 << 8));
pub const EShTargetSpv_1_2: EShTargetLanguageVersion = EShTargetLanguageVersion((1 << 16) | (2 << 8));
pub const EShTargetSpv_1_3: EShTargetLanguageVersion = EShTargetLanguageVersion((1 << 16) | (3 << 8));
pub const EShTargetSpv_1_4: EShTargetLanguageVersion = EShTargetLanguageVersion((1 << 16) | (4 << 8));
pub const EShTargetSpv_1_5: EShTargetLanguageVersion = EShTargetLanguageVersion((1 << 16) | (5 << 8));

/// typedef enum EShExecutable
#[repr(transparent)]
pub struct EShExecutable(c_int);

pub const EShExVertexFragment: EShExecutable = EShExecutable(0);
pub const EShExFragmen: EShExecutable = EShExecutable(1);

/// typedef enum EShOptimizationLevel
#[repr(transparent)]
pub struct EShOptimizationLevel(c_int);

pub const EShOptNoGeneration: EShOptimizationLevel = EShOptimizationLevel(0);
pub const EShOptNone: EShOptimizationLevel = EShOptimizationLevel(1);
pub const EShOptSimple: EShOptimizationLevel = EShOptimizationLevel(2);
pub const EShOptFull: EShOptimizationLevel = EShOptimizationLevel(3);

/// typedef enum EShTextureSamplerTransformMode
#[repr(transparent)]
pub struct EShTextureSamplerTransformMode(c_int);

pub const EShTexSampTransKeep: EShTextureSamplerTransformMode = EShTextureSamplerTransformMode(0);
pub const EShTexSampTransUpgradeTextureRemoveSampler: EShTextureSamplerTransformMode =
    EShTextureSamplerTransformMode(1);

/// typedef enum EShMessages (bitflags)
pub type EShMessages = c_uint;

pub const EShMsgDefault: EShMessages = 0;
pub const EShMsgRelaxedErrors: EShMessages = 1 << 0;
pub const EShMsgSuppressWarnings: EShMessages = 1 << 1;
pub const EShMsgAST: EShMessages = 1 << 2;
pub const EShMsgSpvRules: EShMessages = 1 << 3;
pub const EShMsgVulkanRules: EShMessages = 1 << 4;
pub const EShMsgOnlyPreprocessor: EShMessages = 1 << 5;
pub const EShMsgReadHlsl: EShMessages = 1 << 6;
pub const EShMsgCascadingErrors: EShMessages = 1 << 7;
pub const EShMsgKeepUncalled: EShMessages = 1 << 8;
pub const EShMsgHlslOffsets: EShMessages = 1 << 9;
pub const EShMsgDebugInfo: EShMessages = 1 << 10;
pub const EShMsgHlslEnable16BitTypes: EShMessages = 1 << 11;
pub const EShMsgHlslLegalization: EShMessages = 1 << 12;
pub const EShMsgHlslDX9Compatible: EShMessages = 1 << 13;
pub const EShMsgBuiltinSymbolTable: EShMessages = 1 << 14;

/// typedef enum EShReflectionOptions (bitflags)
pub type EShReflectionOptions = c_uint;

pub const EShReflectionDefault: EShReflectionOptions = 0;
pub const EShReflectionStrictArraySuffix: EShReflectionOptions = 1 << 0;
pub const EShReflectionBasicArraySuffix: EShReflectionOptions = 1 << 1;
pub const EShReflectionIntermediateIO: EShReflectionOptions = 1 << 2;
pub const EShReflectionSeparateBuffers: EShReflectionOptions = 1 << 3;
pub const EShReflectionAllBlockVariables: EShReflectionOptions = 1 << 4;
pub const EShReflectionUnwrapIOBlocks: EShReflectionOptions = 1 << 5;
pub const EShReflectionAllIOVariables: EShReflectionOptions = 1 << 6;
pub const EShReflectionSharedStd140SSBO: EShReflectionOptions = 1 << 7;
pub const EShReflectionSharedStd140UBO: EShReflectionOptions = 1 << 8;

/// enum TResourceType
#[repr(transparent)]
pub struct TResourceType(c_int);

pub const EResSampler: TResourceType = TResourceType(0);
pub const EResTexture: TResourceType = TResourceType(1);
pub const EResImage: TResourceType = TResourceType(2);
pub const EResUbo: TResourceType = TResourceType(3);
pub const EResSsbo: TResourceType = TResourceType(4);
pub const EResUav: TResourceType = TResourceType(5);
pub const EResCount: TResourceType = TResourceType(6);

/// enum TBlockStorageClass
#[repr(transparent)]
pub struct TBlockStorageClass(c_int);

pub const EbsUniform: TBlockStorageClass = TBlockStorageClass(0);
pub const EbsStorageBuffer: TBlockStorageClass = TBlockStorageClass(1);
pub const EbsPushConstant: TBlockStorageClass = TBlockStorageClass(2);
pub const EbsNone: TBlockStorageClass = TBlockStorageClass(3);
pub const EbsCount: TBlockStorageClass = TBlockStorageClass(4);

#[repr(C)]
pub struct Version
{
    major: c_int,
    minor: c_int,
    patch: c_int,
    flavor: *const c_char
}

/// class TShader
#[repr(transparent)]
pub struct TShader(c_void);

/// class TProgram
#[repr(transparent)]
pub struct TProgram(c_void);

/// special struct intended to bridge GlslangToSpv
#[repr(transparent)]
pub struct SpvContext(c_void);

#[repr(C)]
pub struct SpvOptions
{
    generateDebugInfo: bool,
    stripDebugInfo: bool,
    disableOptimizer: bool,
    optimizeSize: bool,
    disassemble: bool,
    validate: bool
}

extern "C" {
    pub fn get_version() -> Version;
    pub fn get_essl_version_string() -> *const c_char;
    pub fn get_khronos_tool_id() -> c_int;

    pub fn initialize_process() -> bool;
    pub fn finalize_process();

    pub fn TShader_create(lang: EShLanguage) -> *const TShader;
    pub fn TShader_destroy(this: *const TShader);
    pub fn TShader_setStrings(this: *const TShader, s: *const *const c_char, n: c_int);
    pub fn TShader_setStringsWithLengths(this: *const TShader, s: *const *const c_char, l: *const c_int, n: c_int);
    pub fn TShader_setStringsWithLengthsAndNames(
        this: *const TShader,
        s: *const *const c_char,
        l: *const c_int,
        names: *const *const c_char,
        n: c_int
    );
    pub fn TShader_setPreamble(this: *const TShader, s: *const c_char);
    pub fn TShader_setEntryPoint(this: *const TShader, entryPoint: *const c_char);
    pub fn TShader_setSourceEntryPoint(this: *const TShader, sourceEntryPointName: *const c_char);
    pub fn TShader_setUniqueId(this: *const TShader, id: c_ulonglong);
    pub fn TShader_setShiftBinding(this: *const TShader, res: TResourceType, base: c_uint);
    pub fn TShader_setShiftBindingForSet(this: *const TShader, res: TResourceType, base: c_uint, set: c_uint);
    pub fn TShader_setAutoMapBindings(this: *const TShader, map: bool);
    pub fn TShader_setAutoMapLocations(this: *const TShader, map: bool);
    pub fn TShader_addUniformLocationOverride(this: *const TShader, name: *const c_char, loc: c_int);
    pub fn TShader_setUniformLocationBase(this: *const TShader, base: c_int);
    pub fn TShader_setInvertY(this: *const TShader, invert: bool);
    pub fn TShader_setNoStorageFormat(this: *const TShader, useUnknownFormat: bool);
    pub fn TShader_setNanMinMaxClamp(this: *const TShader, nanMinMaxClamp: bool);
    pub fn TShader_setTextureSamplerTransformMode(this: *const TShader, mode: EShTextureSamplerTransformMode);
    pub fn TShader_addBlockStorageOverride(this: *const TShader, nameStr: *const c_char, backing: TBlockStorageClass);
    pub fn TShader_setGlobalUniformBlockName(this: *const TShader, name: *const c_char);
    pub fn TShader_setAtomicCounterBlockName(this: *const TShader, name: *const c_char);
    pub fn TShader_setGlobalUniformSet(this: *const TShader, set: c_uint);
    pub fn TShader_setGlobalUniformBinding(this: *const TShader, binding: c_uint);
    pub fn TShader_setAtomicCounterBlockSet(this: *const TShader, set: c_uint);
    pub fn TShader_setAtomicCounterBlockBinding(this: *const TShader, binding: c_uint);
    pub fn TShader_setEnvInput(
        this: *const TShader,
        lang: EShSource,
        stage: EShLanguage,
        client: EShClient,
        version: c_int
    );
    pub fn TShader_setEnvClient(this: *const TShader, client: EShClient, version: EShTargetClientVersion);
    pub fn TShader_setEnvTarget(this: *const TShader, lang: EShTargetLanguage, version: EShTargetLanguageVersion);
    pub fn TShader_getStrings(this: *const TShader, s: *mut *const *const c_char, n: *mut c_int);
    pub fn TShader_setEnvInputVulkanRulesRelaxed(this: *const TShader);
    pub fn TShader_getEnvTargetHlslFunctionality1(this: *const TShader) -> bool;
    pub fn TShader_getEnvInputVulkanRulesRelaxed(this: *const TShader) -> bool;
    pub fn TShader_parse(
        this: *const TShader,
        res: *const limits::TBuiltInResource,
        defaultVersion: c_int,
        defaultProfile: versions::EProfile,
        forceDefaultVersionAndProfile: bool,
        forwardCompatible: bool,
        messages: EShMessages
    ) -> bool;
    pub fn TShader_parse1(
        this: *const TShader,
        res: *const limits::TBuiltInResource,
        defaultVersion: c_int,
        forwardCompatible: bool,
        messages: EShMessages
    ) -> bool;
    pub fn TShader_getInfoLog(this: *const TShader) -> *const c_char;
    pub fn TShader_getInfoDebugLog(this: *const TShader) -> *const c_char;
    pub fn TShader_getStage(this: *const TShader) -> EShLanguage;
    pub fn TShader_getIntermediate(this: *const TShader) -> *const c_void;

    pub fn TProgram_create() -> *const TProgram;
    pub fn TProgram_destroy(this: *const TProgram);
    pub fn TProgram_addShader(this: *const TProgram, shader: *const TShader);
    pub fn TProgram_link(this: *const TProgram, messages: EShMessages) -> bool;
    pub fn TProgram_buildReflection(this: *const TProgram, opts: EShReflectionOptions) -> bool;
    pub fn TProgram_getInfoLog(this: *const TProgram) -> *const c_char;
    pub fn TProgram_getInfoDebugLog(this: *const TProgram) -> *const c_char;
    pub fn TProgram_getIntermediate(this: *const TProgram, stage: EShLanguage) -> *const c_void;
    pub fn TProgram_getPipeIOIndex(this: *const TProgram, name: *const c_char, inOrOut: bool) -> c_int;
    pub fn TProgram_getNumLiveUniformVariables(this: *const TProgram) -> c_int;
    pub fn TProgram_getNumLiveUniformBlocks(this: *const TProgram) -> c_int;
    pub fn TProgram_getNumLiveAttributes(this: *const TProgram) -> c_int;
    pub fn TProgram_getUniformIndex(this: *const TProgram, name: *const c_char) -> c_int;
    pub fn TProgram_getUniformBinding(this: *const TProgram, index: c_int) -> c_int;
    pub fn TProgram_getUniformBlockIndex(this: *const TProgram, index: c_int) -> c_int;
    pub fn TProgram_getUniformType(this: *const TProgram, index: c_int) -> c_int;
    pub fn TProgram_getUniformBufferOffset(this: *const TProgram, index: c_int) -> c_int;
    pub fn TProgram_getUniformArraySize(this: *const TProgram, index: c_int) -> c_int;
    pub fn TProgram_getUniformBlockSize(this: *const TProgram, index: c_int) -> c_int;
    pub fn TProgram_getUniformBlockBinding(this: *const TProgram, index: c_int) -> c_int;
    pub fn TProgram_getUniformBlockCounterIndex(this: *const TProgram, index: c_int) -> c_int;
    pub fn TProgram_getAttributeType(this: *const TProgram, index: c_int) -> c_int;
    pub fn TProgram_getUniformStages(this: *const TProgram, index: c_int) -> EShLanguageMask;
    pub fn TProgram_getAttributeName(this: *const TProgram, index: c_int) -> *const c_char;
    pub fn TProgram_getUniformName(this: *const TProgram, index: c_int) -> *const c_char;
    pub fn TProgram_getUniformBlockName(this: *const TProgram, index: c_int) -> *const c_char;
    pub fn TProgram_dumpReflection(this: *const TProgram);

    pub fn SpvContext_create() -> *const SpvContext;
    pub fn SpvContext_fromGlslang(this: *const SpvContext, intermediate: *const c_void, options: *const SpvOptions);
    pub fn SpvContext_getLog(this: *const SpvContext) -> *const c_char;
    pub fn SpvContext_getData(this: *const SpvContext) -> *const c_uint;
    //TODO: update to c_size_t when https://github.com/rust-lang/rust/issues/88345 is stable
    pub fn SpvContext_getSize(this: *const SpvContext) -> usize;
    pub fn SpvContext_destroy(this: *const SpvContext);
}

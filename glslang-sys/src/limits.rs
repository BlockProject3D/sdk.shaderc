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

use std::os::raw::c_int;

#[repr(C)]
pub struct TLimits
{
    nonInductiveForLoops: bool,
    whileLoops: bool,
    doWhileLoops: bool,
    generalUniformIndexing: bool,
    generalAttributeMatrixVectorIndexing: bool,
    generalVaryingIndexing: bool,
    generalSamplerIndexing: bool,
    generalVariableIndexing: bool,
    generalConstantMatrixVectorIndexing: bool
}

#[repr(C)]
pub struct TBuiltInResource
{
    maxLights: c_int,
    maxClipPlanes: c_int,
    maxTextureUnits: c_int,
    maxTextureCoords: c_int,
    maxVertexAttribs: c_int,
    maxVertexUniformComponents: c_int,
    maxVaryingFloats: c_int,
    maxVertexTextureImageUnits: c_int,
    maxCombinedTextureImageUnits: c_int,
    maxTextureImageUnits: c_int,
    maxFragmentUniformComponents: c_int,
    maxDrawBuffers: c_int,
    maxVertexUniformVectors: c_int,
    maxVaryingVectors: c_int,
    maxFragmentUniformVectors: c_int,
    maxVertexOutputVectors: c_int,
    maxFragmentInputVectors: c_int,
    minProgramTexelOffset: c_int,
    maxProgramTexelOffset: c_int,
    maxClipDistances: c_int,
    maxComputeWorkGroupCountX: c_int,
    maxComputeWorkGroupCountY: c_int,
    maxComputeWorkGroupCountZ: c_int,
    maxComputeWorkGroupSizeX: c_int,
    maxComputeWorkGroupSizeY: c_int,
    maxComputeWorkGroupSizeZ: c_int,
    maxComputeUniformComponents: c_int,
    maxComputeTextureImageUnits: c_int,
    maxComputeImageUniforms: c_int,
    maxComputeAtomicCounters: c_int,
    maxComputeAtomicCounterBuffers: c_int,
    maxVaryingComponents: c_int,
    maxVertexOutputComponents: c_int,
    maxGeometryInputComponents: c_int,
    maxGeometryOutputComponents: c_int,
    maxFragmentInputComponents: c_int,
    maxImageUnits: c_int,
    maxCombinedImageUnitsAndFragmentOutputs: c_int,
    maxCombinedShaderOutputResources: c_int,
    maxImageSamples: c_int,
    maxVertexImageUniforms: c_int,
    maxTessControlImageUniforms: c_int,
    maxTessEvaluationImageUniforms: c_int,
    maxGeometryImageUniforms: c_int,
    maxFragmentImageUniforms: c_int,
    maxCombinedImageUniforms: c_int,
    maxGeometryTextureImageUnits: c_int,
    maxGeometryOutputVertices: c_int,
    maxGeometryTotalOutputComponents: c_int,
    maxGeometryUniformComponents: c_int,
    maxGeometryVaryingComponents: c_int,
    maxTessControlInputComponents: c_int,
    maxTessControlOutputComponents: c_int,
    maxTessControlTextureImageUnits: c_int,
    maxTessControlUniformComponents: c_int,
    maxTessControlTotalOutputComponents: c_int,
    maxTessEvaluationInputComponents: c_int,
    maxTessEvaluationOutputComponents: c_int,
    maxTessEvaluationTextureImageUnits: c_int,
    maxTessEvaluationUniformComponents: c_int,
    maxTessPatchComponents: c_int,
    maxPatchVertices: c_int,
    maxTessGenLevel: c_int,
    maxViewports: c_int,
    maxVertexAtomicCounters: c_int,
    maxTessControlAtomicCounters: c_int,
    maxTessEvaluationAtomicCounters: c_int,
    maxGeometryAtomicCounters: c_int,
    maxFragmentAtomicCounters: c_int,
    maxCombinedAtomicCounters: c_int,
    maxAtomicCounterBindings: c_int,
    maxVertexAtomicCounterBuffers: c_int,
    maxTessControlAtomicCounterBuffers: c_int,
    maxTessEvaluationAtomicCounterBuffers: c_int,
    maxGeometryAtomicCounterBuffers: c_int,
    maxFragmentAtomicCounterBuffers: c_int,
    maxCombinedAtomicCounterBuffers: c_int,
    maxAtomicCounterBufferSize: c_int,
    maxTransformFeedbackBuffers: c_int,
    maxTransformFeedbackInterleavedComponents: c_int,
    maxCullDistances: c_int,
    maxCombinedClipAndCullDistances: c_int,
    maxSamples: c_int,
    maxMeshOutputVerticesNV: c_int,
    maxMeshOutputPrimitivesNV: c_int,
    maxMeshWorkGroupSizeX_NV: c_int,
    maxMeshWorkGroupSizeY_NV: c_int,
    maxMeshWorkGroupSizeZ_NV: c_int,
    maxTaskWorkGroupSizeX_NV: c_int,
    maxTaskWorkGroupSizeY_NV: c_int,
    maxTaskWorkGroupSizeZ_NV: c_int,
    maxMeshViewCountNV: c_int,
    maxDualSourceDrawBuffersEXT: c_int,

    limits: TLimits
}

extern "C" {
    pub fn TBuiltInResource_default() -> *const TBuiltInResource;
}
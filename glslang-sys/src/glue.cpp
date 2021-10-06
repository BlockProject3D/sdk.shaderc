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

#include "../glslang/glslang/Public/ShaderLang.h"

using namespace glslang;

extern "C"
{

Version get_version()
{
    return GetVersion();
}

const char *get_essl_version_string()
{
    return GetEsslVersionString();
}

int get_khronos_tool_id()
{
    return GetKhronosToolId();
}

bool initialize_process()
{
    return InitializeProcess();
}

void finalize_process()
{
    return FinalizeProcess();
}

void *TShader_create(EShLanguage lang)
{
    return new TShader(lang);
}

void TShader_setStrings(void *self, const char *const *s, int n)
{
    auto *shader = (TShader *) self;
    shader->setStrings(s, n);
}

void TShader_setPreamble(void *self, const char *s)
{
    auto *shader = (TShader *) self;
    shader->setPreamble(s);
}

void TShader_setStringsWithLengths(void *self, const char *const *s, const int *l, int n)
{
    auto *shader = (TShader *) self;
    shader->setStringsWithLengths(s, l, n);
}

void
TShader_setStringsWithLengthsAndNames(void *self, const char *const *s, const int *l, const char *const *names, int n)
{
    auto *shader = (TShader *) self;
    shader->setStringsWithLengthsAndNames(s, l, names, n);
}

void TShader_setEntryPoint(void *self, const char *entryPoint)
{
    auto *shader = (TShader *) self;
    shader->setEntryPoint(entryPoint);
}

void TShader_setSourceEntryPoint(void *self, const char *sourceEntryPointName)
{
    auto *shader = (TShader *) self;
    shader->setSourceEntryPoint(sourceEntryPointName);
}

void TShader_setUniqueId(void *self, unsigned long long id)
{
    auto *shader = (TShader *) self;
    shader->setUniqueId(id);
}

void TShader_setShiftBinding(void *self, TResourceType res, unsigned int base)
{
    auto *shader = (TShader *) self;
    shader->setShiftBinding(res, base);
}

void TShader_setShiftBindingForSet(void *self, TResourceType res, unsigned int base, unsigned int set)
{
    auto *shader = (TShader *) self;
    shader->setShiftBindingForSet(res, base, set);
}

void TShader_setAutoMapBindings(void *self, bool map)
{
    auto *shader = (TShader *) self;
    shader->setAutoMapBindings(map);
}

void TShader_setAutoMapLocations(void *self, bool map)
{
    auto *shader = (TShader *) self;
    shader->setAutoMapLocations(map);
}

void TShader_addUniformLocationOverride(void *self, const char *name, int loc)
{
    auto *shader = (TShader *) self;
    shader->addUniformLocationOverride(name, loc);
}

void TShader_setUniformLocationBase(void *self, int base)
{
    auto *shader = (TShader *) self;
    shader->setUniformLocationBase(base);
}

void TShader_setInvertY(void *self, bool invert)
{
    auto *shader = (TShader *) self;
    shader->setInvertY(invert);
}

void TShader_setNoStorageFormat(void *self, bool useUnknownFormat)
{
    auto *shader = (TShader *) self;
    shader->setNoStorageFormat(useUnknownFormat);
}

void TShader_setNanMinMaxClamp(void *self, bool nanMinMaxClamp)
{
    auto *shader = (TShader *) self;
    shader->setNanMinMaxClamp(nanMinMaxClamp);
}

void TShader_setTextureSamplerTransformMode(void *self, EShTextureSamplerTransformMode mode)
{
    auto *shader = (TShader *) self;
    shader->setTextureSamplerTransformMode(mode);
}

void TShader_addBlockStorageOverride(void *self, const char *nameStr, TBlockStorageClass backing)
{
    auto *shader = (TShader *) self;
    shader->addBlockStorageOverride(nameStr, backing);
}

void TShader_setGlobalUniformBlockName(void *self, const char *name)
{
    auto *shader = (TShader *) self;
    shader->setGlobalUniformBlockName(name);
}

void TShader_setAtomicCounterBlockName(void *self, const char *name)
{
    auto *shader = (TShader *) self;
    shader->setAtomicCounterBlockName(name);
}

void TShader_setGlobalUniformSet(void *self, unsigned int set)
{
    auto *shader = (TShader *) self;
    shader->setGlobalUniformSet(set);
}

void TShader_setGlobalUniformBinding(void *self, unsigned int binding)
{
    auto *shader = (TShader *) self;
    shader->setGlobalUniformBinding(binding);
}

void TShader_setAtomicCounterBlockSet(void *self, unsigned int set)
{
    auto *shader = (TShader *) self;
    shader->setAtomicCounterBlockSet(set);
}

void TShader_setAtomicCounterBlockBinding(void *self, unsigned int binding)
{
    auto *shader = (TShader *) self;
    shader->setAtomicCounterBlockBinding(binding);
}

void TShader_setEnvInput(void *self, EShSource lang, EShLanguage envStage, EShClient client, int version)
{
    auto *shader = (TShader *) self;
    shader->setEnvInput(lang, envStage, client, version);
}

void TShader_setEnvClient(void *self, EShClient client, EShTargetClientVersion version)
{
    auto *shader = (TShader *) self;
    shader->setEnvClient(client, version);
}

void TShader_setEnvTarget(void *self, EShTargetLanguage lang, EShTargetLanguageVersion version)
{
    auto *shader = (TShader *) self;
    shader->setEnvTarget(lang, version);
}

void TShader_getStrings(void *self, const char *const **s, int *n)
{
    auto *shader = (TShader *) self;
    shader->getStrings(*s, *n);
}

bool TShader_getEnvTargetHlslFunctionality1(void *self)
{
    auto *shader = (TShader *) self;
    return shader->getEnvTargetHlslFunctionality1();
}

void TShader_setEnvInputVulkanRulesRelaxed(void *self)
{
    auto *shader = (TShader *) self;
    shader->setEnvInputVulkanRulesRelaxed();
}

bool TShader_getEnvInputVulkanRulesRelaxed(void *self)
{
    auto *shader = (TShader *) self;
    return shader->getEnvInputVulkanRulesRelaxed();
}

bool TShader_parse(void *self, const TBuiltInResource *res, int defaultVersion, EProfile defaultProfile,
                   bool forceDefaultVersionAndProfile, bool forwardCompatible, EShMessages messages)
{
    auto *shader = (TShader *) self;
    return shader->parse(res, defaultVersion, defaultProfile, forceDefaultVersionAndProfile, forwardCompatible,
                         messages);
}

bool
TShader_parse1(void *self, const TBuiltInResource *res, int defaultVersion, bool forwardCompatible,
               EShMessages messages)
{
    auto *shader = (TShader *) self;
    return shader->parse(res, defaultVersion, forwardCompatible, messages);
}

const char* TShader_getInfoLog(void *self)
{
    auto *shader = (TShader *) self;
    return shader->getInfoLog();
}

const char* TShader_getInfoDebugLog(void *self)
{
    auto *shader = (TShader *) self;
    return shader->getInfoDebugLog();
}

EShLanguage TShader_getStage(void *self)
{
    auto *shader = (TShader *) self;
    return shader->getStage();
}

void* TShader_getIntermediate(void *self)
{
    auto *shader = (TShader *) self;
    return shader->getIntermediate();
}

void TShader_destroy(void *self)
{
    auto *shader = (TShader *) self;
    delete shader;
}

void *TProgram_create()
{
    return new TProgram();
}

void TProgram_addShader(void *self, void* shader)
{
    auto *prog = (TProgram *) self;
    prog->addShader((TShader *) shader);
}

bool TProgram_link(void *self, EShMessages messages)
{
    auto *prog = (TProgram *) self;
    return prog->link(messages);
}

const char* TProgram_getInfoLog(void *self)
{
    auto *prog = (TProgram *) self;
    return prog->getInfoLog();
}

const char* TProgram_getInfoDebugLog(void *self)
{
    auto *prog = (TProgram *) self;
    return prog->getInfoDebugLog();
}

void* TProgram_getIntermediate(void *self, EShLanguage stage)
{
    auto *prog = (TProgram *) self;
    return prog->getIntermediate(stage);
}

bool TProgram_buildReflection(void *self, int opts)
{
    auto *prog = (TProgram *) self;
    return prog->buildReflection(opts);
}

int TProgram_getPipeIOIndex(void *self, const char *name, const bool inOrOut)
{
    auto *prog = (TProgram *) self;
    return prog->getPipeIOIndex(name, inOrOut);
}

int TProgram_getNumLiveUniformVariables(void *self)
{
    auto *prog = (TProgram *) self;
    return prog->getNumLiveUniformVariables();
}

int TProgram_getNumLiveUniformBlocks(void *self)
{
    auto *prog = (TProgram *) self;
    return prog->getNumLiveUniformBlocks();
}

int TProgram_getNumLiveAttributes(void *self)
{
    auto *prog = (TProgram *) self;
    return prog->getNumLiveAttributes();
}

int TProgram_getUniformIndex(void *self, const char *name)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformIndex(name);
}

const char* TProgram_getUniformName(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformName(index);
}

int TProgram_getUniformBinding(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformBinding(index);
}

EShLanguageMask TProgram_getUniformStages(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformStages(index);
}

int TProgram_getUniformBlockIndex(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformBlockIndex(index);
}

int TProgram_getUniformType(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformType(index);
}

int TProgram_getUniformBufferOffset(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformBufferOffset(index);
}

int TProgram_getUniformArraySize(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformArraySize(index);
}

const char* TProgram_getUniformBlockName(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformBlockName(index);
}

int TProgram_getUniformBlockSize(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformBlockSize(index);
}

int TProgram_getUniformBlockBinding(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformBlockBinding(index);
}

int TProgram_getUniformBlockCounterIndex(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getUniformBlockCounterIndex(index);
}

const char* TProgram_getAttributeName(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getAttributeName(index);
}

int TProgram_getAttributeType(void *self, int index)
{
    auto *prog = (TProgram *) self;
    return prog->getAttributeType(index);
}

void TProgram_dumpReflection(void *self)
{
    auto *prog = (TProgram *) self;
    return prog->dumpReflection();
}

void TProgram_destroy(void *self)
{
    auto *prog = (TProgram *)self;
    delete prog;
}

}

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

use glslang_sys::{
    EShClient,
    EShClientNone,
    EShClientOpenGL,
    EShClientVulkan,
    EShLangFragment,
    EShLangGeometry,
    EShLangTessControl,
    EShLangTessEvaluation,
    EShLangVertex,
    EShLanguage,
    EShTargetClientVersion,
    EShTargetLangNone,
    EShTargetLanguage,
    EShTargetLanguageVersion,
    EShTargetNone,
    EShTargetOpenGL_450,
    EShTargetSpv,
    EShTargetSpv_1_0,
    EShTargetSpv_1_1,
    EShTargetSpv_1_2,
    EShTargetSpv_1_3,
    EShTargetSpv_1_4,
    EShTargetSpv_1_5,
    EShTargetVersionNone,
    EShTargetVulkan_1_0,
    EShTargetVulkan_1_1,
    EShTargetVulkan_1_2
};

#[derive(Copy, Clone, Debug)]
pub enum Stage
{
    Vertex,
    Pixel,
    Geometry,
    Hull,
    Domain
}

impl Stage
{
    pub fn into(self) -> EShLanguage
    {
        match self {
            Stage::Vertex => EShLangVertex,
            Stage::Pixel => EShLangFragment,
            Stage::Geometry => EShLangGeometry,
            Stage::Hull => EShLangTessControl,
            Stage::Domain => EShLangTessEvaluation
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Client
{
    OpenGL,
    Vulkan
}

impl Client
{
    pub fn into(self) -> EShClient
    {
        return match self {
            Client::OpenGL => EShClientOpenGL,
            Client::Vulkan => EShClientVulkan
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ClientVersion
{
    Vulkan10,
    Vulkan11,
    Vulkan12,
    OpenGL450
}

impl ClientVersion
{
    pub fn into(self) -> EShTargetClientVersion
    {
        return match self {
            ClientVersion::Vulkan10 => EShTargetVulkan_1_0,
            ClientVersion::Vulkan11 => EShTargetVulkan_1_1,
            ClientVersion::Vulkan12 => EShTargetVulkan_1_2,
            ClientVersion::OpenGL450 => EShTargetOpenGL_450
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TargetVersion
{
    Spv10,
    Spv11,
    Spv12,
    Spv13,
    Spv14,
    Spv15
}

impl TargetVersion
{
    pub fn into(self) -> EShTargetLanguageVersion
    {
        return match self {
            TargetVersion::Spv10 => EShTargetSpv_1_0,
            TargetVersion::Spv11 => EShTargetSpv_1_1,
            TargetVersion::Spv12 => EShTargetSpv_1_2,
            TargetVersion::Spv13 => EShTargetSpv_1_3,
            TargetVersion::Spv14 => EShTargetSpv_1_4,
            TargetVersion::Spv15 => EShTargetSpv_1_5
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Environment
{
    stage: Stage,
    dialect: Option<Client>,
    dialect_version: i32,
    client: Option<Client>,
    client_version: Option<ClientVersion>,
    spirv: Option<TargetVersion>
}

impl Environment
{
    pub fn new_validation(stage: Stage) -> Environment
    {
        return Environment {
            stage,
            dialect: None,
            dialect_version: 0,
            client: None,
            client_version: None,
            spirv: None
        };
    }

    pub fn new_vulkan(
        stage: Stage,
        dialect: Client,
        dialect_version: Option<i32>,
        client_version: ClientVersion,
        spirv: TargetVersion
    ) -> Environment
    {
        let dv = dialect_version.unwrap_or(300);
        return Environment {
            stage,
            dialect: Some(dialect),
            dialect_version: dv,
            client: Some(Client::Vulkan),
            client_version: Some(client_version),
            spirv: Some(spirv)
        };
    }

    pub fn new_opengl(stage: Stage, dialect: Client, dialect_version: Option<i32>) -> Environment
    {
        let dv = dialect_version.unwrap_or(300);
        return Environment {
            stage,
            dialect: Some(dialect),
            dialect_version: dv,
            client: Some(Client::OpenGL),
            client_version: Some(ClientVersion::OpenGL450),
            spirv: None
        };
    }

    pub fn get_stage(&self) -> EShLanguage
    {
        return self.stage.into();
    }

    pub fn get_dialect(&self) -> EShClient
    {
        if let Some(c) = self.dialect {
            return c.into();
        }
        return EShClientNone;
    }

    pub fn get_client(&self) -> EShClient
    {
        if let Some(c) = self.client {
            return c.into();
        }
        return EShClientNone;
    }

    pub fn get_dialect_version(&self) -> i32
    {
        return self.dialect_version;
    }

    pub fn get_client_version(&self) -> EShTargetClientVersion
    {
        if let Some(c) = self.client_version {
            return c.into();
        }
        return EShTargetVersionNone;
    }

    pub fn get_target_language(&self) -> EShTargetLanguage
    {
        if self.spirv.is_some() {
            return EShTargetSpv;
        }
        return EShTargetNone;
    }

    pub fn get_target_language_version(&self) -> EShTargetLanguageVersion
    {
        if let Some(c) = self.spirv {
            return c.into();
        }
        return EShTargetLangNone;
    }
}

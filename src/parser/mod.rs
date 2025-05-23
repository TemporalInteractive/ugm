#[cfg(feature = "gltf")]
pub(crate) mod gltf;

use crate::texture::TextureCompression;

#[derive(Clone, Copy)]
pub enum MaxTextureResolution {
    Res1024,
    Res2048,
    Res4096,
}

impl MaxTextureResolution {
    pub fn resolution(&self) -> u32 {
        match self {
            Self::Res1024 => 1024,
            Self::Res2048 => 2048,
            Self::Res4096 => 4096,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct ParseOptions {
    pub texture_compression: Option<TextureCompression>,
    pub generate_mips: bool,
    pub max_texture_resolution: Option<MaxTextureResolution>,
    pub merge_duplicate_meshes: bool,
}

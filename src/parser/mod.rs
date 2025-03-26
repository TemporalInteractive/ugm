#[cfg(feature = "gltf")]
mod gltf;

pub use gltf::parse_glb;

use crate::texture::TextureCompression;

#[derive(Default, Clone, Copy)]
pub struct ParseOptions {
    pub texture_compression: Option<TextureCompression>,
}

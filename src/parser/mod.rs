#[cfg(feature = "gltf")]
pub(crate) mod gltf;

use crate::texture::TextureCompression;

#[derive(Default, Clone, Copy)]
pub struct ParseOptions {
    pub texture_compression: Option<TextureCompression>,
    pub generate_mips: bool,
}

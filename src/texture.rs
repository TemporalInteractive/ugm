use half::f16;
use speedy::{Readable, Writable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Readable, Writable)]
pub enum TextureFormat {
    Uncompressed(UncompressedTextureFormat),
    Compressed(CompressedTextureFormat),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureCompression {
    Bc,
    Astc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Readable, Writable)]
pub enum UncompressedTextureFormat {
    R8Unorm,
    Rg8Unorm,
    Rgba8Unorm,
    Rgba32Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Readable, Writable)]
pub enum CompressedTextureFormat {
    Bc4RUnorm,
    Bc5RgUnorm,
    Bc7RgbaUnorm,
    Bc6hRgbUfloat,
}

impl UncompressedTextureFormat {
    pub fn num_channels(&self) -> usize {
        match self {
            Self::R8Unorm => 1,
            Self::Rg8Unorm => 2,
            Self::Rgba8Unorm => 4,
            Self::Rgba32Float => 4,
        }
    }

    pub fn bytes_per_channel(&self) -> usize {
        match self {
            Self::R8Unorm | Self::Rg8Unorm | Self::Rgba8Unorm => size_of::<u8>(),
            Self::Rgba32Float => size_of::<f32>(),
        }
    }

    pub fn bytes_per_row(&self, width: u32) -> usize {
        width as usize * self.num_channels() * self.bytes_per_channel()
    }

    pub fn try_as_compressed(
        &self,
        texture_compression: &TextureCompression,
    ) -> Option<&CompressedTextureFormat> {
        match texture_compression {
            TextureCompression::Bc => match self {
                Self::R8Unorm => Some(&CompressedTextureFormat::Bc4RUnorm),
                Self::Rg8Unorm => Some(&CompressedTextureFormat::Bc5RgUnorm),
                Self::Rgba8Unorm => Some(&CompressedTextureFormat::Bc7RgbaUnorm),
                Self::Rgba32Float => Some(&CompressedTextureFormat::Bc6hRgbUfloat),
            },
            TextureCompression::Astc => todo!("ASTC is not supported yet!"),
        }
    }

    #[cfg(feature = "wgpu")]
    pub fn to_wgpu(&self) -> wgpu::TextureFormat {
        match self {
            Self::R8Unorm => wgpu::TextureFormat::R8Unorm,
            Self::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
            Self::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            _ => panic!("Failed to convert {:?} to wgpu format.", self),
        }
    }
}

impl CompressedTextureFormat {
    pub fn block_size(&self) -> usize {
        match self {
            Self::Bc4RUnorm => 8,
            Self::Bc5RgUnorm | Self::Bc7RgbaUnorm | Self::Bc6hRgbUfloat => 16,
        }
    }

    pub fn bytes_per_row(&self, width: u32) -> usize {
        ((width + 3) / 4) as usize * self.block_size()
    }

    #[cfg(feature = "wgpu")]
    pub fn to_wgpu(&self) -> wgpu::TextureFormat {
        match self {
            Self::Bc4RUnorm => wgpu::TextureFormat::Bc4RUnorm,
            Self::Bc5RgUnorm => wgpu::TextureFormat::Bc5RgUnorm,
            Self::Bc7RgbaUnorm => wgpu::TextureFormat::Bc7RgbaUnorm,
            Self::Bc6hRgbUfloat => wgpu::TextureFormat::Bc6hRgbUfloat,
            _ => panic!("Failed to convert {:?} to wgpu format.", self),
        }
    }
}

pub struct TextureCreateDesc<'a> {
    pub name: Option<&'a str>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct Texture {
    name: String,
    width: u32,
    height: u32,
    format: TextureFormat,
    data: Vec<u8>,
}

impl Texture {
    pub fn new(desc: TextureCreateDesc) -> Self {
        Self {
            name: desc.name.unwrap_or("Unnamed").to_owned(),
            width: desc.width,
            height: desc.height,
            format: desc.format,
            data: desc.data,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn compress(&self, texture_compression: &TextureCompression) -> Option<Self> {
        if let TextureFormat::Uncompressed(uncompressed_format) = self.format() {
            if let Some(compressed_format) =
                uncompressed_format.try_as_compressed(texture_compression)
            {
                let data = match compressed_format {
                    CompressedTextureFormat::Bc4RUnorm => {
                        let surface = intel_tex_2::RSurface {
                            width: self.width,
                            height: self.height,
                            stride: self.width
                                * (uncompressed_format.num_channels()
                                    * uncompressed_format.bytes_per_channel())
                                    as u32,
                            data: &self.data,
                        };

                        intel_tex_2::bc4::compress_blocks(&surface)
                    }
                    CompressedTextureFormat::Bc5RgUnorm => {
                        let surface = intel_tex_2::RgSurface {
                            width: self.width,
                            height: self.height,
                            stride: self.width
                                * (uncompressed_format.num_channels()
                                    * uncompressed_format.bytes_per_channel())
                                    as u32,
                            data: &self.data,
                        };

                        intel_tex_2::bc5::compress_blocks(&surface)
                    }
                    CompressedTextureFormat::Bc6hRgbUfloat => {
                        let f32_data = bytemuck::cast_slice(&self.data);
                        let f16_data: Vec<f16> =
                            f32_data.iter().copied().map(f16::from_f32).collect();

                        let surface = intel_tex_2::RgbaSurface {
                            width: self.width,
                            height: self.height,
                            stride: self.width
                                * (uncompressed_format.num_channels()
                                    * uncompressed_format.bytes_per_channel())
                                    as u32,
                            data: bytemuck::cast_slice(&f16_data),
                        };

                        intel_tex_2::bc6h::compress_blocks(
                            &intel_tex_2::bc6h::very_fast_settings(),
                            &surface,
                        )
                    }
                    CompressedTextureFormat::Bc7RgbaUnorm => {
                        let surface = intel_tex_2::RgbaSurface {
                            width: self.width,
                            height: self.height,
                            stride: self.width
                                * (uncompressed_format.num_channels()
                                    * uncompressed_format.bytes_per_channel())
                                    as u32,
                            data: &self.data,
                        };

                        intel_tex_2::bc7::compress_blocks(
                            &intel_tex_2::bc7::alpha_ultra_fast_settings(),
                            &surface,
                        )
                    }
                };

                return Some(Self {
                    name: self.name.clone(),
                    width: self.width,
                    height: self.height,
                    format: TextureFormat::Compressed(*compressed_format),
                    data,
                });
            }
        }

        None
    }
}

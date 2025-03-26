use speedy::{Readable, Writable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Readable, Writable)]
pub enum TextureFormat {
    Uncompressed(UncompressedTextureFormat),
    Compressed(CompressedTextureFormat),
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

    pub fn try_as_compressed(&self) -> Option<&CompressedTextureFormat> {
        match self {
            Self::R8Unorm => Some(&CompressedTextureFormat::Bc4RUnorm),
            Self::Rg8Unorm => Some(&CompressedTextureFormat::Bc5RgUnorm),
            Self::Rgba8Unorm => Some(&CompressedTextureFormat::Bc7RgbaUnorm),
            Self::Rgba32Float => Some(&CompressedTextureFormat::Bc6hRgbUfloat),
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
}

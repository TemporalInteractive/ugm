use half::f16;
use image::DynamicImage;
use speedy::{Readable, Writable};
use uuid::Uuid;

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

impl TextureFormat {
    pub fn bytes_per_row(&self, width: u32) -> usize {
        match self {
            Self::Uncompressed(format) => format.bytes_per_row(width),
            Self::Compressed(format) => format.bytes_per_row(width),
        }
    }

    #[cfg(feature = "wgpu")]
    pub fn to_wgpu(&self) -> wgpu::TextureFormat {
        match self {
            Self::Uncompressed(format) => format.to_wgpu(),
            Self::Compressed(format) => format.to_wgpu(),
        }
    }
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
        width.div_ceil(4) as usize * self.block_size()
    }

    #[cfg(feature = "wgpu")]
    pub fn to_wgpu(&self) -> wgpu::TextureFormat {
        match self {
            Self::Bc4RUnorm => wgpu::TextureFormat::Bc4RUnorm,
            Self::Bc5RgUnorm => wgpu::TextureFormat::Bc5RgUnorm,
            Self::Bc7RgbaUnorm => wgpu::TextureFormat::Bc7RgbaUnorm,
            Self::Bc6hRgbUfloat => wgpu::TextureFormat::Bc6hRgbUfloat,
        }
    }
}

pub struct TextureCreateDesc<'a> {
    pub name: Option<&'a str>,
    pub image: image::DynamicImage,
    pub mips: bool,
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct Texture {
    name: String,
    uuid: Uuid,
    width: u32,
    height: u32,
    mip_count: u32,
    format: TextureFormat,
    data: Vec<Vec<u8>>,
    uv_offset: [f32; 2],
    uv_scale: [f32; 2],
}

impl Texture {
    pub fn new(desc: TextureCreateDesc) -> Self {
        let mut mipmaps = vec![desc.image];
        while mipmaps.last().unwrap().width() > 1 && mipmaps.last().unwrap().height() > 1 {
            let next_width = (mipmaps.last().unwrap().width() / 2).max(1);
            let next_height = (mipmaps.last().unwrap().height() / 2).max(1);

            let next = mipmaps.last().unwrap().resize_exact(
                next_width,
                next_height,
                image::imageops::FilterType::CatmullRom,
            );

            mipmaps.push(next);
        }

        #[allow(clippy::needless_range_loop)]
        for i in 0..mipmaps.len() {
            let converted_mip = match mipmaps[i] {
                DynamicImage::ImageRgba16(_) => {
                    Some(DynamicImage::ImageRgba8(mipmaps[i].to_rgba8()))
                }
                DynamicImage::ImageRgb16(_) => {
                    Some(DynamicImage::ImageRgba8(mipmaps[i].to_rgba8()))
                }
                DynamicImage::ImageLumaA16(_) => {
                    Some(DynamicImage::ImageLumaA8(mipmaps[i].to_luma_alpha8()))
                }
                DynamicImage::ImageLuma16(_) => {
                    Some(DynamicImage::ImageLuma8(mipmaps[i].to_luma8()))
                }
                DynamicImage::ImageRgb8(_) => Some(DynamicImage::ImageRgba8(mipmaps[i].to_rgba8())),
                _ => None,
            };

            if let Some(converted_mip) = converted_mip {
                mipmaps[i] = converted_mip;
            }
        }

        let format = match &mipmaps[0] {
            DynamicImage::ImageRgba32F(_) => {
                TextureFormat::Uncompressed(UncompressedTextureFormat::Rgba32Float)
            }
            DynamicImage::ImageRgba8(_) => {
                TextureFormat::Uncompressed(UncompressedTextureFormat::Rgba8Unorm)
            }
            DynamicImage::ImageLumaA8(_) => {
                TextureFormat::Uncompressed(UncompressedTextureFormat::Rg8Unorm)
            }
            DynamicImage::ImageLuma8(_) => {
                TextureFormat::Uncompressed(UncompressedTextureFormat::R8Unorm)
            }
            _ => panic!(),
        };

        let mut data = Vec::new();
        for mip in &mipmaps {
            data.push(mip.as_bytes().to_vec());
        }

        Self {
            name: desc.name.unwrap_or("Unnamed").to_owned(),
            uuid: Uuid::new_v4(),
            width: mipmaps[0].width(),
            height: mipmaps[0].height(),
            mip_count: mipmaps.len() as u32,
            format,
            data,
            uv_offset: desc.uv_offset,
            uv_scale: desc.uv_scale,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
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

    pub fn data(&self) -> &[Vec<u8>] {
        &self.data
    }

    pub fn uv_offset(&self) -> [f32; 2] {
        self.uv_offset
    }

    pub fn uv_scale(&self) -> [f32; 2] {
        self.uv_scale
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
                            data: &self.data[0],
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
                            data: &self.data[0],
                        };

                        intel_tex_2::bc5::compress_blocks(&surface)
                    }
                    CompressedTextureFormat::Bc6hRgbUfloat => {
                        let f32_data = bytemuck::cast_slice(&self.data[0]);
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
                            data: &self.data[0],
                        };

                        intel_tex_2::bc7::compress_blocks(
                            &intel_tex_2::bc7::alpha_ultra_fast_settings(),
                            &surface,
                        )
                    }
                };

                return Some(Self {
                    name: self.name.clone(),
                    uuid: Uuid::new_v4(),
                    width: self.width,
                    height: self.height,
                    mip_count: self.mip_count,
                    format: TextureFormat::Compressed(*compressed_format),
                    data: vec![data],
                    uv_offset: self.uv_offset,
                    uv_scale: self.uv_scale,
                });
            }
        }

        None
    }

    #[cfg(feature = "wgpu")]
    pub fn create_wgpu_texture(
        &self,
        usage: wgpu::TextureUsages,
        srgb: bool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let mut format = self.format.to_wgpu();
        if srgb {
            format = format.add_srgb_suffix();
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: self.mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: usage | wgpu::TextureUsages::COPY_DST,
            label: Some(&self.name),
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });

        let mut mip_width = self.width;
        let mut mip_height = self.height;
        for i in 0..self.mip_count {
            let bytes_per_row = self.format.bytes_per_row(mip_width);

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: i,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.data[i as usize],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row as u32),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: mip_width,
                    height: mip_height,
                    depth_or_array_layers: 1,
                },
            );

            mip_width = (mip_width / 2).max(1);
            mip_height = (mip_height / 2).max(1);
        }

        (texture, texture_view)
    }
}

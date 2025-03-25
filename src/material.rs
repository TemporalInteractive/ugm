use speedy::{Readable, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct Material {
    pub index: Option<usize>,

    pub color: [f32; 3],
    pub color_texture: Option<u32>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<u32>,
    pub normal_scale: f32,
    pub normal_texture: Option<u32>,
    pub emission: [f32; 3],
    pub emission_texture: Option<u32>,

    pub absorption: [f32; 3],
    pub transmission: f32,
    pub transmission_texture: Option<u32>,
    pub eta: f32,

    pub subsurface: f32,
    pub specular: f32,
    pub specular_tint: [f32; 3],
    pub anisotropic: f32,

    pub sheen: f32,
    pub sheen_texture: Option<u32>,
    pub sheen_tint: [f32; 3],
    pub sheen_tint_texture: Option<u32>,

    pub clearcoat: f32,
    pub clearcoat_texture: Option<u32>,
    pub clearcoat_roughness: f32,
    pub clearcoat_roughness_texture: Option<u32>,
    pub clearcoat_normal_texture: Option<u32>,

    pub is_opaque: bool,
    pub alpha_cutoff: f32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            index: None,
            color: [1.0; 3],
            color_texture: None,
            metallic: 0.0,
            roughness: 0.5,
            metallic_roughness_texture: None,
            normal_scale: 1.0,
            normal_texture: None,
            emission: [0.0; 3],
            emission_texture: None,

            absorption: [0.0; 3],
            transmission: 0.0,
            transmission_texture: None,
            eta: 1.0 / 1.5,

            subsurface: 0.0,
            specular: 0.0,
            specular_tint: [1.0; 3],
            anisotropic: 0.0,

            sheen: 0.0,
            sheen_texture: None,
            sheen_tint: [1.0; 3],
            sheen_tint_texture: None,

            clearcoat: 0.0,
            clearcoat_texture: None,
            clearcoat_roughness: 0.0,
            clearcoat_roughness_texture: None,
            clearcoat_normal_texture: None,

            is_opaque: true,
            alpha_cutoff: 0.0,
        }
    }
}

impl Material {
    pub fn is_emissive(&self) -> bool {
        self.emission[0] > 0.0 || self.emission[1] > 0.0 || self.emission[2] > 0.0
    }
}

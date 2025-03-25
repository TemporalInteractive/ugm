use material::Material;
use mesh::Mesh;
use speedy::{Readable, Writable};
use texture::Texture;

pub mod parser;

mod material;
mod mesh;
mod packing;
mod texture;

#[derive(Debug, Clone, Readable, Writable)]
pub struct ModelNode {
    pub name: String,
    pub transform: [f32; 16],
    pub mesh_idx: Option<u32>,

    pub child_node_indices: Vec<u32>,
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct Model {
    pub root_node_indices: Vec<u32>,
    pub nodes: Vec<ModelNode>,

    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
}

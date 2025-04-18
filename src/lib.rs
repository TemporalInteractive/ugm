use glam::Mat4;
use material::Material;
use mesh::Mesh;
use speedy::{Readable, Writable};
use texture::Texture;

pub mod material;
pub mod mesh;
pub mod packing;
pub mod parser;
pub mod texture;

pub use speedy;

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
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],

    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
}

impl Model {
    #[cfg(feature = "gltf")]
    pub fn parse_glb(data: &[u8], opt: parser::ParseOptions) -> anyhow::Result<Self> {
        parser::gltf::parse_glb(data, opt)
    }

    pub fn traverse_nodes<F>(&self, root_transform: Mat4, mut callback: F)
    where
        F: FnMut(&ModelNode, Mat4),
    {
        for root_node in &self.root_node_indices {
            self.traverse_nodes_recursive(*root_node, root_transform, &mut callback);
        }
    }

    fn traverse_nodes_recursive<F>(&self, node: u32, parent_transform: Mat4, callback: &mut F)
    where
        F: FnMut(&ModelNode, Mat4),
    {
        let node = &self.nodes[node as usize];
        let transform = parent_transform * Mat4::from_cols_array(&node.transform);

        callback(node, transform);

        for child_node in &node.child_node_indices {
            self.traverse_nodes_recursive(*child_node, transform, callback);
        }
    }
}

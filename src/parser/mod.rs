#[cfg(feature = "gltf")]
mod gltf;

pub use gltf::parse_glb;

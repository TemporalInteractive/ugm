# 🌐 Universal Game Model

[![ugm crate](https://img.shields.io/crates/v/ugm.svg)](https://crates.io/crates/ugm)

Universal game model is a minimal crate designed as a high performance model format. Loading times can be very slow when repeatedly parsing a glTF model into a custom engine representation over and over each time a game is loaded up. Ugm solves this by allowing you to convert a glTF model into a ugm model once at bake time, after which it can be loaded directly as is with a 10x performance improvement!

## Features
- [x] glTF parsing
- [x] Bc texture compression
- [x] Normal & tangent generation
- [x] Vertex packing
- [X] Mipmap generation
- [ ] Astc texture compression

## Usage
Serialization and deserialization are handled by [speedy](https://crates.io/crates/speedy). For more info please look at their [docs](https://docs.rs/speedy/0.8.7/speedy/).
```rust
use ugm::{parser::ParseOptions, texture::TextureCompression, Model};

// Get the bytes of your model, more commonly you'd read it from a file
let gltf_model_bytes: &[u8; 0] = include_bytes!("ToyCar.glb");

// Parse a glTF model into a ugm model, this is relatively slow and shouldn't happen each time the application is launched
let ugm_model: Model = Model::parse_glb(
    gltf_model_bytes,
    ParseOptions {
        // Optional texture compression, this will increase parse duration
        texture_compression: Some(TextureCompression::Bc),
    },
)
.expect("Failed to parse glTF model.");

// Serialize ugm model into bytes
let ugm_model_bytes: Vec<u8> = ugm_model.write_to_vec().unwrap();

// Here you'd want to write out the serialized ugm model, which can then be loaded the next times the application is launched...

// Deserialize bytes into a ugm model
let ugm_model = Model::read_from_buffer(&ugm_model_bytes).unwrap();
```

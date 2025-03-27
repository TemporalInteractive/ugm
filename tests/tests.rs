#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use speedy::{Readable, Writable};
    use ugm::{parser::ParseOptions, texture::TextureCompression, Model};

    #[test]
    fn parse_gltf() {
        let model_bytes = include_bytes!("ToyCar.glb");
        let _ = Model::parse_glb(model_bytes, ParseOptions::default()).unwrap();
    }

    #[test]
    fn serialize_model() {
        let model_bytes = include_bytes!("ToyCar.glb");
        let model = Model::parse_glb(model_bytes, ParseOptions::default()).unwrap();

        let serialized = model.write_to_vec().unwrap();
        let compression_rate = serialized.len() as f32 / model_bytes.len() as f32;
        println!("Compression rate: {}", compression_rate);
    }

    #[test]
    fn deserialize_model() {
        let model_bytes = include_bytes!("ToyCar.glb");
        let model = Model::parse_glb(model_bytes, ParseOptions::default()).unwrap();

        let serialized = model.write_to_vec().unwrap();
        let _ = Model::read_from_buffer(&serialized).unwrap();
    }

    #[test]
    fn bc_texture_compression() {
        let model_bytes = include_bytes!("ToyCar.glb");
        let model = Model::parse_glb(
            model_bytes,
            ParseOptions {
                texture_compression: Some(TextureCompression::Bc),
            },
        )
        .unwrap();

        let serialized = model.write_to_vec().unwrap();
        let compression_rate = serialized.len() as f32 / model_bytes.len() as f32;
        println!("Compression rate: {}", compression_rate);
    }

    #[test]
    fn parse_vs_deserialize() {
        let model_bytes = include_bytes!("ToyCar.glb");

        let now = std::time::Instant::now();
        let model = Model::parse_glb(model_bytes, ParseOptions::default()).unwrap();
        let parse_duration = now.elapsed().as_secs_f32();

        let serialized = model.write_to_vec().unwrap();

        let now = std::time::Instant::now();
        let _ = black_box(Model::read_from_buffer(&serialized).unwrap());
        let deserialize_duration = now.elapsed().as_secs_f32();

        println!(
            "Parse duration: {}s | Deserialize duration: {}s",
            parse_duration, deserialize_duration
        );
        println!("Speedup rate: {}", parse_duration / deserialize_duration);
    }
}

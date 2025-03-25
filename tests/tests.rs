#[cfg(test)]
mod tests {
    use speedy::Writable;
    use ugm::*;

    #[test]
    fn parse_gltf() {
        let model_bytes = include_bytes!("ToyCar.glb");
        let model = ugm::parser::parse_glb(model_bytes).unwrap();
    }

    #[test]
    fn parse_serialize_model() {
        let model_bytes = include_bytes!("ToyCar.glb");
        let model = ugm::parser::parse_glb(model_bytes).unwrap();

        let serialized = model.write_to_vec().unwrap();
        let compression_rate = model_bytes.len() as f32 / serialized.len() as f32;
        println!("Compression rate: {}", compression_rate)
    }
}

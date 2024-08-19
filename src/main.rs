use protobuf_zod_converter::parser::parse_proto_file;
use crate::generator::{generate_zod_schema, GeneratorConfig};

fn main() {
    let proto_file = parse_proto_file("example.proto").expect("Failed to parse Protobuf file");
    let config = GeneratorConfig::default();

    match generate_zod_schema(&proto_file, config) {
        Ok(schema) => println!("Generated schema:\n{}", schema),
        Err(e) => eprintln!("Generation failed: {:?}", e),
    }
}
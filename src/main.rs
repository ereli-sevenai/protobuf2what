use protobuf_to_zod::parser::parse_proto_file;
use std::fs;
use std::path::PathBuf;

fn main() {
    let mut proto_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    proto_path.push("files");
    proto_path.push("simple.proto");

    let proto_content = fs::read_to_string(proto_path).expect("Failed to read the proto file");

    let _proto_file = parse_proto_file(&proto_content).expect("Failed to parse Protobuf file");

    println!("Success!");
}

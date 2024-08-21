use protobuf_to_zod::parser::parse_proto_file;
use std::fs;

fn main() {
    let proto_content =
        fs::read_to_string("../files/simple.proto").expect("Failed to read the proto file");

    let _proto_file = parse_proto_file(&proto_content).expect("Failed to parse Protobuf file");

    println!("Success!");
}

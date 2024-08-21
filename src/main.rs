use protobuf_to_zod::parser::parse_proto_file;

fn main() {
    let file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/files/simple.proto");
    let _proto_file = parse_proto_file(file_path).expect("Failed to parse Protobuf file");

    println!("Success!")
}

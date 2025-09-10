fn main() { println\!("{:?}", protobuf_to_zod::parser::parse_proto_file("syntax = \"proto3\";
import \"test.proto\";").err()); }

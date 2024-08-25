use crate::parser::ast::ProtoFile;

pub trait Visitor {
    fn visit_proto_file(&mut self, proto_file: &ProtoFile);
    // Other visit methods...
}

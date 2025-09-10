#[cfg(test)]
mod tests {
    use crate::parser::{ast::{Enum, Field, FieldLabel, FieldType, Message, NumberValue, ProtoFile, Syntax}, parse_proto_file, ImportKind};

    #[test]
    fn test_parse_simple_proto() {
        let proto_content = r#"
            syntax = "proto3";
            package test;

            message Simple {
                string name = 1;
                int32 id = 2;
                bool active = 3;
            }
        "#;

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());

        let proto = result.unwrap();
        assert_eq!(proto.syntax, Syntax::Proto3);
        assert_eq!(proto.package, Some("test".to_string()));
        assert_eq!(proto.messages.len(), 1);
        
        let message = &proto.messages[0];
        assert_eq!(message.name, "Simple");
        assert_eq!(message.fields.len(), 3);

        let fields = &message.fields;
        assert_eq!(fields[0].name, "name");
        assert_eq!(fields[0].typ, FieldType::String);

        assert_eq!(fields[1].name, "id");
        assert_eq!(fields[1].typ, FieldType::Int32);

        assert_eq!(fields[2].name, "active");
        assert_eq!(fields[2].typ, FieldType::Bool);
    }

    #[test]
    fn test_parse_enum() {
        let proto_content = r#"
            syntax = "proto3";
            
            enum Status {
                UNKNOWN = 0;
                ACTIVE = 1;
                INACTIVE = 2;
            }
        "#;

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());

        let proto = result.unwrap();
        assert_eq!(proto.enums.len(), 1);
        
        let enum_def = &proto.enums[0];
        assert_eq!(enum_def.name, "Status");
        assert_eq!(enum_def.values.len(), 3);

        let values = &enum_def.values;
        assert_eq!(values[0].name, "UNKNOWN");
        assert_eq!(values[1].name, "ACTIVE");
        assert_eq!(values[2].name, "INACTIVE");
    }

    #[test]
    fn test_parse_nested_message() {
        let proto_content = r#"
            syntax = "proto3";
            
            message Outer {
                string name = 1;
                
                message Inner {
                    int32 id = 1;
                    string value = 2;
                }
                
                Inner inner = 2;
            }
        "#;

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());

        let proto = result.unwrap();
        assert_eq!(proto.messages.len(), 1);
        
        let outer = &proto.messages[0];
        assert_eq!(outer.name, "Outer");
        assert_eq!(outer.fields.len(), 2);
        assert_eq!(outer.nested_messages.len(), 1);

        let inner = &outer.nested_messages[0];
        assert_eq!(inner.name, "Inner");
        assert_eq!(inner.fields.len(), 2);
    }

    #[test]
    fn test_parse_repeated_field() {
        let proto_content = r#"
            syntax = "proto3";
            
            message WithRepeated {
                repeated string tags = 1;
                repeated int32 counts = 2;
            }
        "#;

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());

        let proto = result.unwrap();
        assert_eq!(proto.messages.len(), 1);
        
        let message = &proto.messages[0];
        assert_eq!(message.fields.len(), 2);
        
        assert_eq!(message.fields[0].label, FieldLabel::Repeated);
        assert_eq!(message.fields[0].typ, FieldType::String);
        
        assert_eq!(message.fields[1].label, FieldLabel::Repeated);
        assert_eq!(message.fields[1].typ, FieldType::Int32);
    }

    #[test]
    fn test_parse_map_field() {
        let proto_content = r#"
            syntax = "proto3";
            
            message WithMap {
                map<string, int32> counts = 1;
                map<string, string> metadata = 2;
            }
        "#;

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());

        let proto = result.unwrap();
        assert_eq!(proto.messages.len(), 1);
        
        let message = &proto.messages[0];
        assert_eq!(message.fields.len(), 2);
        
        if let FieldType::Map(key_type, value_type) = &message.fields[0].typ {
            assert_eq!(**key_type, FieldType::String);
            assert_eq!(**value_type, FieldType::Int32);
        } else {
            panic!("Expected map type for field counts");
        }
        
        if let FieldType::Map(key_type, value_type) = &message.fields[1].typ {
            assert_eq!(**key_type, FieldType::String);
            assert_eq!(**value_type, FieldType::String);
        } else {
            panic!("Expected map type for field metadata");
        }
    }

    #[test]
    fn test_parse_with_options() {
        let proto_content = r#"
            syntax = "proto3";
            
            option java_package = "com.example.protos";
            option go_package = "example.com/protos";
            
            message WithOptions {
                string name = 1;
                option deprecated = true;
            }
        "#;

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());

        let proto = result.unwrap();
        assert_eq!(proto.options.len(), 2);
        assert_eq!(proto.messages.len(), 1);
        
        let message = &proto.messages[0];
        assert_eq!(message.options.len(), 1);
    }

    #[test]
    fn test_parse_with_imports() {
        let proto_content = "syntax = \"proto3\";\n\n\
            import \"google/protobuf/timestamp.proto\";\n\
            import public \"other.proto\";\n\
            import weak \"legacy.proto\";\n\n\
            message WithImports {\n\
                string name = 1;\n\
                google.protobuf.Timestamp created_at = 2;\n\
            }\n";

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());

        let proto = result.unwrap();
        assert_eq!(proto.imports.len(), 3);
        
        assert_eq!(proto.imports[0].path, "google/protobuf/timestamp.proto");
        assert_eq!(proto.imports[0].kind, ImportKind::Default);
        
        assert_eq!(proto.imports[1].path, "other.proto");
        assert_eq!(proto.imports[1].kind, ImportKind::Public);
        
        assert_eq!(proto.imports[2].path, "legacy.proto");
        assert_eq!(proto.imports[2].kind, ImportKind::Weak);
    }

    #[test]
    fn test_parse_with_comments() {
        let proto_content = r#"
            // File comment
            syntax = "proto3";
            
            // Message comment
            message WithComments {
                // Field comment
                string name = 1;
                
                // This is a comment about the ID
                int32 id = 2; // Inline comment
                
                bool active = 3; // Another inline comment
            }
            
            // Enum comment
            enum Status {
                // Value comment
                UNKNOWN = 0; // Inline value comment
                ACTIVE = 1;
                INACTIVE = 2;
            }
        "#;

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_reserved() {
        let proto_content = r#"
            syntax = "proto3";
            
            message WithReserved {
                string name = 1;
                
                reserved 2, 15, 9 to 11;
                reserved "foo", "bar";
                
                int32 id = 20;
            }
        "#;

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());

        let proto = result.unwrap();
        let message = &proto.messages[0];
        
        assert_eq!(message.reserved.len(), 5);
    }

    #[test]
    fn test_parse_service() {
        let proto_content = r#"
            syntax = "proto3";
            
            service TestService {
                rpc DoSomething(RequestMessage) returns (ResponseMessage);
                rpc StreamData(RequestMessage) returns (stream ResponseMessage);
                rpc UploadData(stream RequestMessage) returns (ResponseMessage);
                rpc Chat(stream RequestMessage) returns (stream ResponseMessage);
            }
            
            message RequestMessage {
                string query = 1;
            }
            
            message ResponseMessage {
                string result = 1;
            }
        "#;

        let result = parse_proto_file(proto_content);
        assert!(result.is_ok());

        let proto = result.unwrap();
        assert_eq!(proto.services.len(), 1);
        
        let service = &proto.services[0];
        assert_eq!(service.name, "TestService");
        assert_eq!(service.methods.len(), 4);
        
        let methods = &service.methods;
        
        // Regular RPC
        assert_eq!(methods[0].name, "DoSomething");
        assert!(!methods[0].client_streaming);
        assert!(!methods[0].server_streaming);
        
        // Server streaming
        assert_eq!(methods[1].name, "StreamData");
        assert!(!methods[1].client_streaming);
        assert!(methods[1].server_streaming);
        
        // Client streaming
        assert_eq!(methods[2].name, "UploadData");
        assert!(methods[2].client_streaming);
        assert!(!methods[2].server_streaming);
        
        // Bidirectional streaming
        assert_eq!(methods[3].name, "Chat");
        assert!(methods[3].client_streaming);
        assert!(methods[3].server_streaming);
    }
}
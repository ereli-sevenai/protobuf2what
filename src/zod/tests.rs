#[cfg(test)]
mod tests {
    use crate::parser::parse_proto_file;
    use crate::zod::parser::ZodAnnotationParser;
    use crate::zod::generator::{ZodGenerator, ZodGeneratorConfig, ImportStyle};
    
    #[test]
    fn test_extract_zod_annotations() {
        let line = r#"string username = 1; // @zod { min: 3, max: 50, description: "User's name" }"#;
        let result = ZodAnnotationParser::extract_zod_annotations(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), r#"{ min: 3, max: 50, description: "User's name" }"#);
    }
    
    #[test]
    fn test_extract_version() {
        let proto_content = r#"syntax = "proto3"; // @zod-version: 1.0"#;
        let proto_file = parse_proto_file(proto_content).unwrap();
        let zod_metadata = ZodAnnotationParser::parse_file(&proto_file, proto_content);
        
        assert_eq!(zod_metadata.file.version, Some("1.0".to_string()));
    }
    
    #[test]
    fn test_parse_message_annotation() {
        let proto_content = r#"
            syntax = "proto3";
            
            message User // @zod { description: "User model" }
            {
                string username = 1;
            }
        "#;
        
        let proto_file = parse_proto_file(proto_content).unwrap();
        let zod_metadata = ZodAnnotationParser::parse_file(&proto_file, proto_content);
        
        assert!(zod_metadata.messages.contains_key("User"));
        let user_metadata = &zod_metadata.messages["User"];
        assert_eq!(user_metadata.message.description, Some("User model".to_string()));
    }
    
    #[test]
    fn test_parse_field_annotations() {
        let proto_content = r#"
            syntax = "proto3";
            
            message User {
                string username = 1; // @zod { min: 3, max: 50, description: "User's name" }
                string email = 2; // @zod { email: true }
                int32 age = 3; // @zod { min: 0, max: 120 }
            }
        "#;
        
        let proto_file = parse_proto_file(proto_content).unwrap();
        let zod_metadata = ZodAnnotationParser::parse_file(&proto_file, proto_content);
        
        assert!(zod_metadata.messages.contains_key("User"));
        let user_metadata = &zod_metadata.messages["User"];
        
        // Check username field
        assert!(user_metadata.fields.contains_key("username"));
        let username_metadata = &user_metadata.fields["username"];
        assert_eq!(username_metadata.min, Some(3));
        assert_eq!(username_metadata.max, Some(50));
        assert_eq!(username_metadata.description, Some("User's name".to_string()));
        
        // Check email field
        assert!(user_metadata.fields.contains_key("email"));
        let email_metadata = &user_metadata.fields["email"];
        assert_eq!(email_metadata.email, Some(true));
        
        // Check age field
        assert!(user_metadata.fields.contains_key("age"));
        let age_metadata = &user_metadata.fields["age"];
        assert_eq!(age_metadata.min, Some(0));
        assert_eq!(age_metadata.max, Some(120));
    }
    
    #[test]
    fn test_generator_basic() {
        let proto_content = r#"
            syntax = "proto3";
            
            message User {
                string username = 1;
                int32 age = 2;
                bool active = 3;
            }
        "#;
        
        let proto_file = parse_proto_file(proto_content).unwrap();
        let zod_metadata = ZodAnnotationParser::parse_file(&proto_file, proto_content);
        
        let config = ZodGeneratorConfig {
            import_style: ImportStyle::Named,
            single_file: true,
            output_dir: "generated".to_string(),
        };
        
        let generator = ZodGenerator::new(zod_metadata, config);
        let result = generator.generate(&proto_file);
        
        assert_eq!(result.len(), 1);
        
        // The generated code should contain User schema
        let content = result.values().next().unwrap();
        assert!(content.contains("export const User = z.object({"));
        assert!(content.contains("username: z.string()"));
        assert!(content.contains("age: z.number().int()"));
        assert!(content.contains("active: z.boolean()"));
        assert!(content.contains("export type User = z.infer<typeof User>;"));
    }
    
    #[test]
    fn test_generator_with_annotations() {
        let proto_content = r#"
            syntax = "proto3"; // @zod-version: 1.0
            
            message User // @zod { description: "User model" }
            {
                string username = 1; // @zod { min: 3, max: 50, description: "User's name" }
                string email = 2; // @zod { email: true }
                int32 age = 3; // @zod { min: 0, max: 120 }
                repeated string tags = 4; // @zod { array: { min: 1, max: 10 } }
            }
            
            enum Role // @zod { description: "User roles" }
            {
                USER = 0;
                ADMIN = 1;
                MODERATOR = 2;
            }
        "#;
        
        let proto_file = parse_proto_file(proto_content).unwrap();
        let zod_metadata = ZodAnnotationParser::parse_file(&proto_file, proto_content);
        
        let config = ZodGeneratorConfig {
            import_style: ImportStyle::Named,
            single_file: true,
            output_dir: "generated".to_string(),
        };
        
        let generator = ZodGenerator::new(zod_metadata, config);
        let result = generator.generate(&proto_file);
        
        assert_eq!(result.len(), 1);
        
        // The generated code should contain User schema with validations
        let content = result.values().next().unwrap();
        assert!(content.contains("// Generated from Protocol Buffer version 1.0"));
        assert!(content.contains("export const User = z.object({"));
        assert!(content.contains("username: z.string().min(3).max(50).describe(\"User's name\")"));
        assert!(content.contains("email: z.string().email()"));
        assert!(content.contains("age: z.number().int().min(0).max(120)"));
        assert!(content.contains("tags: z.string().array()"));
        assert!(content.contains("export const Role = z.enum(['USER', 'ADMIN', 'MODERATOR']).describe(\"User roles\")"));
    }
}
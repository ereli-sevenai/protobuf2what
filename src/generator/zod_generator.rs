
use std::collections::HashMap;

use crate::parser::ast::{ProtoFile, Message, Field, FieldType, Enum, EnumValue, FieldLabel};

/// Configuration options for Zod schema generation
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Whether to generate TypeScript types alongside Zod schemas
    pub generate_types: bool,
    /// Whether to include comments from the Protobuf file in the generated schemas
    pub include_comments: bool,
    /// Indentation string to use (e.g., "  " for two spaces, "\t" for tab)
    pub indentation: String,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        GeneratorConfig {
            generate_types: true,
            include_comments: true,
            indentation: "  ".to_string(),
        }
    }
}

/// Errors that can occur during Zod schema generation
#[derive(Debug, Error)]
pub enum GeneratorError {
    SchemaGenerationError(String),
    UnsupportedFeature(String),
}

pub struct ZodGenerator<T, K, V> {
    config: GeneratorConfig,
    statements: Vec<T>,
    type_map: HashMap<K,V>
}

impl ZodGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        let mut generator = ZodGenerator {
            statements: Vec::new(),
            type_map: HashMap::new(),
            config,
        };

        // actual work

        generator
    }

    pub fn generate(&mut self, proto_file: &ProtoFile) -> String {
        self.generate_enums(&proto_file.enums);
        self.generate_messages(&proto_file.messages);

        // codegen::generate_typescript(&self.statements, ExportType::Default)

        return String::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Syntax, Import, ImportKind};
    use crate::generator::zod_generator::ZodGenerator;

    #[test]
    fn test_generate_zod_schema() {
        let proto_file = ProtoFile {
            syntax: Syntax::Proto3,
            package: Some("example.package".to_string()),
            imports: vec![Import {
                path: "google/protobuf/timestamp.proto".to_string(),
                kind: ImportKind::Default,
            }],
            options: Vec::new(),
            messages: vec![
                Message {
                    name: "Person".to_string(),
                    fields: vec![
                        Field {
                            name: "name".to_string(),
                            number: 1,
                            label: FieldLabel::Required,
                            typ: FieldType::String,
                            options: Vec::new(),
                        },
                        Field {
                            name: "age".to_string(),
                            number: 2,
                            label: FieldLabel::Optional,
                            typ: FieldType::Int32,
                            options: Vec::new(),
                        },
                        Field {
                            name: "emails".to_string(),
                            number: 3,
                            label: FieldLabel::Repeated,
                            typ: FieldType::String,
                            options: Vec::new(),
                        },
                    ],
                    oneofs: Vec::new(),
                    nested_messages: Vec::new(),
                    nested_enums: Vec::new(),
                    options: Vec::new(),
                    reserved: Vec::new(),
                }
            ],
            enums: vec![
                Enum {
                    name: "Gender".to_string(),
                    values: vec![
                        EnumValue {
                            name: "UNKNOWN".to_string(),
                            number: 0,
                            options: Vec::new(),
                        },
                        EnumValue {
                            name: "MALE".to_string(),
                            number: 1,
                            options: Vec::new(),
                        },
                        EnumValue {
                            name: "FEMALE".to_string(),
                            number: 2,
                            options: Vec::new(),
                        },
                    ],
                    options: Vec::new(),
                }
            ],
            services: Vec::new(),
        };

        let config = GeneratorConfig {
            generate_types: true,
            include_comments: false,
            indentation: "    ".to_string(), // 4 spaces
        };

        let mut generator = ZodGenerator::new(config);
        let zod_schema = generator.generate(&proto_file);

        assert!(zod_schema.contains("import { z } from \"zod\";"));
        assert!(zod_schema.contains("const GenderSchema = z.enum([\"UNKNOWN\", \"MALE\", \"FEMALE\"]);"));
        assert!(zod_schema.contains("const PersonSchema = z.object({"));
        assert!(zod_schema.contains("name: z.string(),"));
        assert!(zod_schema.contains("age: z.number().optional(),"));
        assert!(zod_schema.contains("emails: z.array(z.string()),"));
    }
}
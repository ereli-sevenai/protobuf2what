//! Zod schema generator module for Protobuf to Zod converter
//! 
//! This module is responsible for generating Zod schemas from Protobuf AST.

mod zod_generator;

use crate::parser::ast::ProtoFile;
use crate::parser::error;
use zod_generator::ZodGenerator;

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
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("Failed to generate Zod schema: {0}")]
    SchemaGenerationError(String),
    #[error("Unsupported Protobuf feature: {0}")]
    UnsupportedFeature(String),
}

/// Generate Zod schemas from a Protobuf AST
///
/// This function takes a `ProtoFile` AST and generates corresponding Zod schemas.
///
/// # Arguments
///
/// * `proto_file` - The Protobuf AST to generate schemas from
/// * `config` - Configuration options for the generator
///
/// # Returns
///
/// A `Result` containing the generated Zod schemas as a `String` if successful,
/// or a `GeneratorError` if generation fails.
pub fn generate_zod_schema(
    proto_file: &ProtoFile,
    config: GeneratorConfig,
) -> Result<String, GeneratorError> {
    let mut generator = ZodGenerator::new(config);
    generator.generate(proto_file)
        .map_err(|e| GeneratorError::SchemaGenerationError(e.to_string()))
}

/// Generate Zod schemas from multiple Protobuf ASTs
///
/// This function takes multiple `ProtoFile` ASTs and generates corresponding Zod schemas.
/// It's useful when dealing with multiple .proto files that may have interdependencies.
///
/// # Arguments
///
/// * `proto_files` - A slice of Protobuf ASTs to generate schemas from
/// * `config` - Configuration options for the generator
///
/// # Returns
///
/// A `Result` containing the generated Zod schemas as a `String` if successful,
/// or a `GeneratorError` if generation fails.
pub fn generate_zod_schemas(
    proto_files: &[ProtoFile],
    config: GeneratorConfig,
) -> Result<String, GeneratorError> {
    let mut generator = ZodGenerator::new(config);
    let mut output = String::new();

    for proto_file in proto_files {
        let schema = generator.generate(proto_file)
            .map_err(|e| GeneratorError::SchemaGenerationError(e.to_string()))?;
        output.push_str(&schema);
        output.push_str("\n\n");
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Syntax, Message, Field, FieldType, FieldLabel};

    #[test]
    fn test_generate_zod_schema() {
        let proto_file = ProtoFile {
            syntax: Syntax::Proto3,
            package: Some("example".to_string()),
            imports: Vec::new(),
            messages: vec![
                Message {
                    name: "Person".to_string(),
                    fields: vec![
                        Field {
                            name: "name".to_string(),
                            number: 1,
                            label: FieldLabel::Optional,
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
                    ],
                    oneofs: Vec::new(),
                    nested_messages: Vec::new(),
                    nested_enums: Vec::new(),
                    options: Vec::new(),
                    reserved: Vec::new(),
                },
            ],
            enums: Vec::new(),
            services: Vec::new(),
            options: Vec::new(),
        };

        let config = GeneratorConfig::default();
        let result = generate_zod_schema(&proto_file, config);

        assert!(result.is_ok(), "Failed to generate Zod schema: {:?}", result.err());
        let schema = result.unwrap();
        assert!(schema.contains("z.object({"));
        assert!(schema.contains("name: z.string().optional()"));
        assert!(schema.contains("age: z.number().int().optional()"));
    }
}
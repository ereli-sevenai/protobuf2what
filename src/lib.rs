//! Protobuf to Zod Schema Converter
//!
//! This library provides functionality to convert Protocol Buffer (protobuf) definitions
//! to Zod schemas. It includes a parser for protobuf files, an intermediate representation,
//! and a generator for Zod schemas.

use std::fs;
use std::path::Path;

pub mod parser;
pub mod intermediate;
pub mod generator;
pub mod visitor;

use parser::parse_proto_file;
use intermediate::ProtoFile;
use generator::generate_zod_schema;

/// Errors that can occur during the conversion process
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    FileReadError(#[from] std::io::Error),
    ParseError(String),
    GenerationError(String),
}

/// Convert a protobuf file to a Zod schema
///
/// This function reads a protobuf file, parses it, and generates a corresponding Zod schema.
///
/// # Arguments
///
/// * `input_path` - Path to the input protobuf file
/// * `output_path` - Optional path to write the output Zod schema. If None, the schema is returned as a String.
///
/// # Returns
///
/// If `output_path` is None, returns the generated Zod schema as a String.
/// If `output_path` is Some, writes the schema to the specified file and returns ().
///
/// # Errors
///
/// Returns a `ConversionError` if any step of the process fails.
pub fn convert_proto_to_zod<P: AsRef<Path>>(
    input_path: P,
    output_path: Option<P>,
) -> Result<Option<String>, ConversionError> {
    // Read the protobuf file
    let proto_content = fs::read_to_string(input_path)?;

    // Parse the protobuf content
    let proto_file = parse_proto_file(&proto_content).map_err(ConversionError::ParseError)?;

    // Generate the Zod schema
    let zod_schema = generate_zod_schema(&proto_file).map_err(ConversionError::GenerationError)?;

    // Write to file or return as string
    match output_path {
        Some(path) => {
            fs::write(path, &zod_schema)?;
            Ok(None)
        }
        None => Ok(Some(zod_schema)),
    }
}

/// Parse a protobuf file and return the intermediate representation
///
/// This function is useful if you want to perform custom operations on the parsed protobuf
/// before generating a Zod schema.
///
/// # Arguments
///
/// * `input_path` - Path to the input protobuf file
///
/// # Returns
///
/// Returns the parsed `ProtoFile` representing the intermediate representation of the protobuf.
///
/// # Errors
///
/// Returns a `ConversionError` if reading or parsing the file fails.
pub fn parse_proto_file_from_path<P: AsRef<Path>>(input_path: P) -> Result<ProtoFile, ConversionError> {
    let proto_content = fs::read_to_string(input_path)?;
    parse_proto_file(&proto_content).map_err(ConversionError::ParseError)
}

/// Generate a Zod schema from a parsed protobuf file
///
/// This function is useful if you have already parsed a protobuf file and want to generate
/// a Zod schema from it.
///
/// # Arguments
///
/// * `proto_file` - The parsed `ProtoFile` representing the protobuf
///
/// # Returns
///
/// Returns the generated Zod schema as a String.
///
/// # Errors
///
/// Returns a `ConversionError` if generation fails.
pub fn generate_zod_schema_from_proto(proto_file: &ProtoFile) -> Result<String, ConversionError> {
    generate_zod_schema(proto_file).map_err(ConversionError::GenerationError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_proto_to_zod() {
        let input = r#"
            syntax = "proto3";
            message Test {
                string name = 1;
                int32 age = 2;
            }
        "#;
        let proto_file = parse_proto_file(input).unwrap();
        let zod_schema = generate_zod_schema(&proto_file).unwrap();
        assert!(zod_schema.contains("z.object"));
        assert!(zod_schema.contains("name: z.string()"));
        assert!(zod_schema.contains("age: z.number()"));
    }
}
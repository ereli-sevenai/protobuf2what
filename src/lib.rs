//! Protobuf to Zod Schema Converter
//!
//! This library provides functionality to convert Protocol Buffer (protobuf) definitions
//! to Zod schemas. It includes a parser for protobuf files, an intermediate representation,
//! and a generator for Zod schemas.

use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

pub mod intermediate;
pub mod parser;
pub mod visitor;

/// Errors that can occur during the conversion process
#[derive(Debug)]
pub enum ConversionError {
    FileReadError(std::io::Error),
    ParseError(String),
    GenerationError(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::FileReadError(err) => write!(f, "File read error: {}", err),
            ConversionError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConversionError::GenerationError(msg) => write!(f, "Generation error: {}", msg),
        }
    }
}

impl Error for ConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConversionError::FileReadError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ConversionError {
    fn from(err: std::io::Error) -> Self {
        ConversionError::FileReadError(err)
    }
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
    // Read the content from the input file
    let input_content =
        fs::read_to_string(input_path.as_ref()).map_err(ConversionError::FileReadError)?;

    // Placeholder: Process the content (conversion logic here)
    let processed_content = format!("Processed: {}", input_content);

    if let Some(out_path) = output_path {
        // Write the processed content to the specified output path
        fs::write(out_path.as_ref(), &processed_content).map_err(ConversionError::FileReadError)?;
        Ok(None)
    } else {
        // Return the processed content as a string
        Ok(Some(processed_content))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_proto_file;

    #[test]
    fn test_convert_proto_to_zod() {
        let input = r#"
            syntax = "proto3";
            message Test {
                string name = 1;
                int32 age = 2;
            }
        "#;
        let _proto_file = parse_proto_file(input).unwrap();
    }
}

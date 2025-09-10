use crate::parser::ast::{Enum, Field, Message, ProtoFile};
use crate::zod::metadata::{ZodFileMetadata, ZodMessageMetadata, ZodMetadata};
use log::{debug, warn};
use regex::Regex;
use serde_json::from_str;
use lazy_static::lazy_static;

/// Parses Zod annotations from Protocol Buffer comments
pub struct ZodAnnotationParser;

lazy_static! {
    static ref VERSION_RE: Regex = Regex::new(r"@zod-version:\s*([0-9.]+)").unwrap();
    static ref ANNOTATION_RE: Regex = Regex::new(r"@zod\s*(\{.+?\})").unwrap();
}

impl ZodAnnotationParser {
    /// Parse all Zod annotations in a proto file and return structured metadata
    pub fn parse_file(proto_file: &ProtoFile, source: &str) -> ZodFileMetadata {
        let mut file_metadata = ZodFileMetadata::default();
        
        // Parse file-level comments (version, global options)
        Self::parse_file_level_comments(source, &mut file_metadata);
        
        // Parse message-level annotations
        for message in &proto_file.messages {
            let message_metadata = Self::parse_message(message, source);
            file_metadata.messages.insert(message.name.clone(), message_metadata);
        }
        
        // Parse enum-level annotations
        for enum_def in &proto_file.enums {
            let enum_metadata = Self::parse_enum(enum_def, source);
            file_metadata.enums.insert(enum_def.name.clone(), enum_metadata);
        }
        
        file_metadata
    }
    
    /// Parse file-level comments for Zod annotations
    fn parse_file_level_comments(source: &str, file_metadata: &mut ZodFileMetadata) {
        // Extract version from comments like: syntax = "proto3"; // @zod-version: 1.0
        if let Some(captures) = VERSION_RE.captures(source) {
            if let Some(version) = captures.get(1) {
                file_metadata.file.version = Some(version.as_str().to_string());
                debug!("Found Zod version: {}", version.as_str());
            }
        }
    }
    
    /// Parse message-level and field-level annotations
    fn parse_message(message: &Message, source: &str) -> ZodMessageMetadata {
        let mut message_metadata = ZodMessageMetadata::default();
        
        // Find message definition in source code to extract comment annotations
        // This is a simplified approach - in practice you'd need to use source locations from the parser
        let message_pattern = format!(r"message\s+{}\s*//\s*@zod\s*(\{{.*?\}})", message.name);
        let message_re = Regex::new(&message_pattern).unwrap();
        
        if let Some(captures) = message_re.captures(source) {
            if let Some(annotation) = captures.get(1) {
                if let Some(metadata) = Self::parse_json_metadata(annotation.as_str()) {
                    message_metadata.message = metadata;
                }
            }
        }
        
        // Parse field-level annotations
        for field in &message.fields {
            let field_metadata = Self::parse_field(field, source);
            message_metadata.fields.insert(field.name.clone(), field_metadata);
        }
        
        message_metadata
    }
    
    /// Parse enum-level annotations
    fn parse_enum(enum_def: &Enum, source: &str) -> ZodMetadata {
        let mut enum_metadata = ZodMetadata::new();
        
        // Find enum definition in source code to extract comment annotations
        let enum_pattern = format!(r"enum\s+{}\s*//\s*@zod\s*(\{{.*?\}})", enum_def.name);
        let enum_re = Regex::new(&enum_pattern).unwrap();
        
        if let Some(captures) = enum_re.captures(source) {
            if let Some(annotation) = captures.get(1) {
                if let Some(metadata) = Self::parse_json_metadata(annotation.as_str()) {
                    enum_metadata = metadata;
                }
            }
        }
        
        enum_metadata
    }
    
    /// Parse field-level annotations
    fn parse_field(field: &Field, source: &str) -> ZodMetadata {
        let mut field_metadata = ZodMetadata::new();
        
        // Find field definition in source code to extract comment annotations
        let field_pattern = format!(r"{}\s*=\s*\d+.*?//\s*@zod\s*(\{{.*?\}})", 
            field.name);
        let field_re = Regex::new(&field_pattern).unwrap();
        
        if let Some(captures) = field_re.captures(source) {
            if let Some(annotation) = captures.get(1) {
                if let Some(metadata) = Self::parse_json_metadata(annotation.as_str()) {
                    field_metadata = metadata;
                }
            }
        }
        
        field_metadata
    }
    
    /// Parse a JSON metadata string into a ZodMetadata struct
    fn parse_json_metadata(json_str: &str) -> Option<ZodMetadata> {
        // Add quotes around the keys to make it valid JSON
        let mut json_with_quotes = String::new();
        let re = Regex::new(r"(\w+)\s*:").unwrap();
        let fixed_json = re.replace_all(json_str, "\"$1\":");
        
        // Fix trailing comma if present
        let fixed_json = fixed_json.replace(",}", "}");
        
        // Fix any nested object key quotes
        let re_nested = Regex::new(r"(\{)\s*(\w+)\s*:").unwrap();
        let fixed_json = re_nested.replace_all(&fixed_json, "$1\"$2\":");
        
        debug!("Original JSON: {}", json_str);
        debug!("Fixed JSON: {}", fixed_json);
        
        match from_str::<ZodMetadata>(&fixed_json) {
            Ok(metadata) => Some(metadata),
            Err(err) => {
                warn!("Failed to parse Zod metadata: {} - Input: {}", err, json_str);
                None
            }
        }
    }
    
    /// Extract all @zod annotations from a line of text
    pub fn extract_zod_annotations(line: &str) -> Option<String> {
        ANNOTATION_RE.captures(line).map(|captures| {
            captures.get(1).map_or("", |m| m.as_str()).to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_zod_annotations() {
        let line = "string username = 1; // @zod { min: 3, max: 50 }";
        let annotation = ZodAnnotationParser::extract_zod_annotations(line);
        assert_eq!(annotation, Some("{ min: 3, max: 50 }".to_string()));
    }
    
    #[test]
    fn test_extract_version() {
        let source = r#"syntax = "proto3"; // @zod-version: 1.0"#;
        let mut file_metadata = ZodFileMetadata::default();
        ZodAnnotationParser::parse_file_level_comments(source, &mut file_metadata);
        assert_eq!(file_metadata.file.version, Some("1.0".to_string()));
    }
}
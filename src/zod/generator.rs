use crate::parser::ast::{Enum, Field, FieldLabel, FieldType, Message, ProtoFile};
use crate::zod::metadata::{ZodFileMetadata, ZodMetadata};
use std::collections::HashMap;
use std::fmt::Write;

/// Generator for Zod schemas from Protocol Buffer definitions
pub struct ZodGenerator {
    /// Metadata extracted from Zod annotations
    metadata: ZodFileMetadata,
    
    /// Import style configuration (default, named, or namespace)
    import_style: ImportStyle,
    
    /// Whether to generate a single file or multiple files
    single_file: bool,
    
    /// Output directory for generated files
    output_dir: String,
}

/// Supported import styles for Zod
pub enum ImportStyle {
    /// import z from 'zod'
    Default,
    
    /// import { z } from 'zod'
    Named,
    
    /// import * as z from 'zod'
    Namespace,
}

/// Configuration for Zod generator
pub struct ZodGeneratorConfig {
    /// Import style for Zod
    pub import_style: ImportStyle,
    
    /// Whether to generate a single file or multiple files
    pub single_file: bool,
    
    /// Output directory for generated files
    pub output_dir: String,
}

impl Default for ZodGeneratorConfig {
    fn default() -> Self {
        ZodGeneratorConfig {
            import_style: ImportStyle::Named,
            single_file: true,
            output_dir: "generated".to_string(),
        }
    }
}

impl ZodGenerator {
    /// Create a new ZodGenerator with the given metadata and configuration
    pub fn new(metadata: ZodFileMetadata, config: ZodGeneratorConfig) -> Self {
        ZodGenerator {
            metadata,
            import_style: config.import_style,
            single_file: config.single_file,
            output_dir: config.output_dir,
        }
    }
    
    /// Generate Zod schemas for a Protocol Buffer file
    pub fn generate(&self, proto_file: &ProtoFile) -> HashMap<String, String> {
        let mut generated_files = HashMap::new();
        
        if self.single_file {
            let file_content = self.generate_single_file(proto_file);
            let filename = self.get_output_filename(proto_file);
            generated_files.insert(filename, file_content);
        } else {
            // Generate separate files for each message and enum
            // This would be implemented for multi-file output
            unimplemented!("Multi-file generation not yet implemented");
        }
        
        generated_files
    }
    
    /// Generate a single TypeScript file with all Zod schemas
    fn generate_single_file(&self, proto_file: &ProtoFile) -> String {
        let mut content = String::new();
        
        // Add imports
        writeln!(content, "{}", self.generate_imports()).unwrap();
        writeln!(content).unwrap();
        
        // Add file-level comments if any
        if let Some(ref version) = self.metadata.file.version {
            writeln!(content, "// Generated from Protocol Buffer version {}", version).unwrap();
        }
        writeln!(content).unwrap();
        
        // Generate enums
        for enum_def in &proto_file.enums {
            writeln!(content, "{}", self.generate_enum(enum_def)).unwrap();
            writeln!(content).unwrap();
        }
        
        // Generate messages
        for message in &proto_file.messages {
            writeln!(content, "{}", self.generate_message(message)).unwrap();
            writeln!(content).unwrap();
        }
        
        content
    }
    
    /// Generate the import statement based on the configured style
    fn generate_imports(&self) -> String {
        match self.import_style {
            ImportStyle::Default => "import z from 'zod';".to_string(),
            ImportStyle::Named => "import { z } from 'zod';".to_string(),
            ImportStyle::Namespace => "import * as z from 'zod';".to_string(),
        }
    }
    
    /// Generate a Zod enum definition
    fn generate_enum(&self, enum_def: &Enum) -> String {
        let mut content = String::new();
        
        // Get metadata for this enum if available
        let enum_metadata = self.metadata.enums.get(&enum_def.name).cloned()
            .unwrap_or_default();
        
        // Add description comment if available
        if let Some(ref description) = enum_metadata.description {
            writeln!(content, "/**\n * {}\n */", description).unwrap();
        }
        
        // Start enum definition
        write!(content, "export const {} = z.enum([", enum_def.name).unwrap();
        
        // Add enum values
        let enum_values: Vec<String> = enum_def.values.iter()
            .map(|value| format!("'{}'", value.name))
            .collect();
            
        write!(content, "{}])", enum_values.join(", ")).unwrap();
        
        // Add metadata constraints if available
        self.apply_metadata_constraints(&mut content, &enum_metadata);
        
        // Close the statement
        writeln!(content, ";").unwrap();
        
        // Add type alias
        writeln!(content, "export type {} = z.infer<typeof {}>;", 
            enum_def.name, enum_def.name).unwrap();
        
        content
    }
    
    /// Generate a Zod message definition
    fn generate_message(&self, message: &Message) -> String {
        let mut content = String::new();
        
        // Get metadata for this message if available
        let message_metadata = self.metadata.messages.get(&message.name).cloned()
            .unwrap_or_default();
        
        // Add description comment if available
        if let Some(ref description) = message_metadata.message.description {
            writeln!(content, "/**\n * {}\n */", description).unwrap();
        }
        
        // Start message definition
        writeln!(content, "export const {} = z.object({{", message.name).unwrap();
        
        // Add fields
        for field in &message.fields {
            writeln!(content, "{},", self.generate_field(field, &message_metadata.fields))
                .unwrap();
        }
        
        // Close object definition
        write!(content, "}})").unwrap();
        
        // Apply message-level metadata constraints if available
        self.apply_metadata_constraints(&mut content, &message_metadata.message);
        
        // Close the statement
        writeln!(content, ";").unwrap();
        
        // Add type alias
        writeln!(content, "export type {} = z.infer<typeof {}>;", 
            message.name, message.name).unwrap();
        
        content
    }
    
    /// Generate a Zod field definition
    fn generate_field(&self, field: &Field, field_metadatas: &HashMap<String, ZodMetadata>) -> String {
        let mut content = String::new();
        
        // Get metadata for this field if available
        let field_metadata = field_metadatas.get(&field.name).cloned()
            .unwrap_or_default();
        
        // Add field name
        write!(content, "  {}: ", field.name).unwrap();
        
        // Generate the field type
        let field_type = match &field.typ {
            FieldType::Double | FieldType::Float => "z.number()".to_string(),
            FieldType::Int32 | FieldType::Int64 |
            FieldType::UInt32 | FieldType::UInt64 |
            FieldType::SInt32 | FieldType::SInt64 |
            FieldType::Fixed32 | FieldType::Fixed64 |
            FieldType::SFixed32 | FieldType::SFixed64 => "z.number().int()".to_string(),
            FieldType::Bool => "z.boolean()".to_string(),
            FieldType::String => "z.string()".to_string(),
            FieldType::Bytes => "z.string()".to_string(), // Bytes represented as base64 strings
            FieldType::MessageOrEnum(ref type_name) => format!("{}", type_name),
            FieldType::Map(ref key_type, ref value_type) => {
                // Maps are represented as records
                format!("z.record({})", self.type_to_zod_type(value_type))
            }
        };
        
        write!(content, "{}", field_type).unwrap();
        
        // Apply field-level metadata constraints
        self.apply_metadata_constraints(&mut content, &field_metadata);
        
        // Handle repeated fields (arrays)
        if field.label == FieldLabel::Repeated {
            write!(content, ".array()").unwrap();
            
            // Apply array-specific constraints if available
            if let Some(ref array_constraints) = field_metadata.array {
                if array_constraints.contains_key("min") {
                    if let Some(min) = array_constraints.get("min").and_then(|v| v.as_u64()) {
                        write!(content, ".min({})", min).unwrap();
                    }
                }
                if array_constraints.contains_key("max") {
                    if let Some(max) = array_constraints.get("max").and_then(|v| v.as_u64()) {
                        write!(content, ".max({})", max).unwrap();
                    }
                }
            }
        }
        
        // Handle optional fields
        let is_optional = field.label == FieldLabel::Optional || 
            field_metadata.optional.unwrap_or(false);
            
        if is_optional {
            write!(content, ".optional()").unwrap();
        }
        
        content
    }
    
    /// Apply metadata constraints to a Zod schema
    fn apply_metadata_constraints(&self, content: &mut String, metadata: &ZodMetadata) {
        // Apply min/max constraints
        if let Some(min) = metadata.min {
            write!(content, ".min({})", min).unwrap();
        }
        if let Some(max) = metadata.max {
            write!(content, ".max({})", max).unwrap();
        }
        
        // Apply email validation
        if metadata.email.unwrap_or(false) {
            write!(content, ".email()").unwrap();
        }
        
        // Apply URL validation
        if metadata.url.unwrap_or(false) {
            write!(content, ".url()").unwrap();
        }
        
        // Apply regex pattern
        if let Some(ref regex) = metadata.regex {
            write!(content, ".regex(new RegExp(\"{}\"))", regex).unwrap();
        }
        
        // Apply description
        if let Some(ref description) = metadata.description {
            write!(content, ".describe(\"{}\")", description).unwrap();
        }
        
        // Apply default value
        if let Some(ref default) = metadata.default {
            write!(content, ".default({})", default).unwrap();
        }
    }
    
    /// Convert a FieldType to its corresponding Zod type
    fn type_to_zod_type(&self, field_type: &FieldType) -> String {
        match field_type {
            FieldType::Double | FieldType::Float => "z.number()".to_string(),
            FieldType::Int32 | FieldType::Int64 |
            FieldType::UInt32 | FieldType::UInt64 |
            FieldType::SInt32 | FieldType::SInt64 |
            FieldType::Fixed32 | FieldType::Fixed64 |
            FieldType::SFixed32 | FieldType::SFixed64 => "z.number().int()".to_string(),
            FieldType::Bool => "z.boolean()".to_string(),
            FieldType::String => "z.string()".to_string(),
            FieldType::Bytes => "z.string()".to_string(), // Bytes represented as base64 strings
            FieldType::MessageOrEnum(ref type_name) => type_name.clone(),
            FieldType::Map(ref _key_type, ref value_type) => {
                // Maps are represented as records
                format!("z.record({})", self.type_to_zod_type(value_type))
            }
        }
    }
    
    /// Get the output filename for a proto file
    fn get_output_filename(&self, proto_file: &ProtoFile) -> String {
        let package_name = proto_file.package.as_deref().unwrap_or("default");
        let sanitized_name = package_name.replace('.', "_");
        format!("{}.ts", sanitized_name)
    }
}
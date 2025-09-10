use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents Zod validation metadata extracted from Protocol Buffer comments
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZodMetadata {
    /// Version specified at the file level, e.g., `// @zod-version: 1.0`
    pub version: Option<String>,
    
    /// Description for documentation
    pub description: Option<String>,
    
    /// Minimum length/value constraint
    pub min: Option<i64>,
    
    /// Maximum length/value constraint
    pub max: Option<i64>,
    
    /// Email validation (for string fields)
    pub email: Option<bool>,
    
    /// URL validation (for string fields)
    pub url: Option<bool>,
    
    /// Regular expression pattern
    pub regex: Option<String>,
    
    /// Default value
    pub default: Option<Value>,
    
    /// Mark field as optional
    pub optional: Option<bool>,
    
    /// Array validation
    pub array: Option<HashMap<String, Value>>,
    
    /// Custom validations not covered by built-in options
    pub custom: Option<HashMap<String, Value>>,
}

/// Metadata for an entire Protocol Buffer file
#[derive(Debug, Clone, Default)]
pub struct ZodFileMetadata {
    /// File-level metadata
    pub file: ZodMetadata,
    
    /// Metadata for messages, keyed by message name
    pub messages: HashMap<String, ZodMessageMetadata>,
    
    /// Metadata for enums, keyed by enum name
    pub enums: HashMap<String, ZodMetadata>,
}

/// Metadata for a Protocol Buffer message
#[derive(Debug, Clone, Default)]
pub struct ZodMessageMetadata {
    /// Message-level metadata
    pub message: ZodMetadata,
    
    /// Metadata for fields, keyed by field name
    pub fields: HashMap<String, ZodMetadata>,
}

impl ZodMetadata {
    /// Creates a new empty ZodMetadata
    pub fn new() -> Self {
        ZodMetadata::default()
    }
    
    /// Merge another metadata object into this one
    pub fn merge(&mut self, other: &ZodMetadata) {
        if let Some(ref v) = other.version {
            self.version = Some(v.clone());
        }
        if let Some(ref v) = other.description {
            self.description = Some(v.clone());
        }
        if let Some(v) = other.min {
            self.min = Some(v);
        }
        if let Some(v) = other.max {
            self.max = Some(v);
        }
        if let Some(v) = other.email {
            self.email = Some(v);
        }
        if let Some(v) = other.url {
            self.url = Some(v);
        }
        if let Some(ref v) = other.regex {
            self.regex = Some(v.clone());
        }
        if let Some(ref v) = other.default {
            self.default = Some(v.clone());
        }
        if let Some(v) = other.optional {
            self.optional = Some(v);
        }
        if let Some(ref v) = other.array {
            let mut new_array = v.clone();
            if let Some(ref mut existing) = self.array {
                for (key, value) in new_array.drain() {
                    existing.insert(key, value);
                }
            } else {
                self.array = Some(new_array);
            }
        }
        if let Some(ref v) = other.custom {
            let mut new_custom = v.clone();
            if let Some(ref mut existing) = self.custom {
                for (key, value) in new_custom.drain() {
                    existing.insert(key, value);
                }
            } else {
                self.custom = Some(new_custom);
            }
        }
    }
}
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use crate::zod::generator::ImportStyle;

/// Configuration for the Zod schema generator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Output directory for generated files
    pub output_dir: String,
    
    /// Target language for schema generation
    #[serde(default)]
    pub target: TargetLanguage,
    
    /// TypeScript-specific configuration
    #[serde(default)]
    pub typescript: TypeScriptConfig,
    
    /// Python-specific configuration
    #[serde(default)]
    pub python: PythonConfig,
    
    /// Whether to create output directories if they don't exist
    #[serde(default = "default_true")]
    pub create_dirs: bool,
    
    /// Whether to override existing files
    #[serde(default = "default_true")]
    pub override_files: bool,
}

/// Target language for schema generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TargetLanguage {
    /// Generate TypeScript/Zod schemas
    TypeScript,
    
    /// Generate Python/Pydantic schemas
    Python,
}

impl Default for TargetLanguage {
    fn default() -> Self {
        TargetLanguage::TypeScript
    }
}

/// TypeScript-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScriptConfig {
    /// Import style for Zod
    #[serde(default)]
    pub import_style: TsImportStyle,
    
    /// Whether to generate a single file or multiple files
    #[serde(default = "default_true")]
    pub single_file: bool,
    
    /// Whether to generate type aliases
    #[serde(default = "default_true")]
    pub generate_types: bool,
    
    /// Extension for generated files (default: .ts)
    #[serde(default = "default_ts_extension")]
    pub file_extension: String,
}

/// Python-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    /// Whether to use dataclasses instead of Pydantic models
    #[serde(default)]
    pub use_dataclasses: bool,
    
    /// Whether to generate a single file or multiple files
    #[serde(default = "default_true")]
    pub single_file: bool,
    
    /// Extension for generated files (default: .py)
    #[serde(default = "default_py_extension")]
    pub file_extension: String,
}

/// Import style for TypeScript
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TsImportStyle {
    /// import z from 'zod'
    Default,
    
    /// import { z } from 'zod'
    Named,
    
    /// import * as z from 'zod'
    Namespace,
}

impl Default for TsImportStyle {
    fn default() -> Self {
        TsImportStyle::Named
    }
}

impl Default for TypeScriptConfig {
    fn default() -> Self {
        TypeScriptConfig {
            import_style: TsImportStyle::default(),
            single_file: default_true(),
            generate_types: default_true(),
            file_extension: default_ts_extension(),
        }
    }
}

impl Default for PythonConfig {
    fn default() -> Self {
        PythonConfig {
            use_dataclasses: false,
            single_file: default_true(),
            file_extension: default_py_extension(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            output_dir: "generated".to_string(),
            target: TargetLanguage::default(),
            typescript: TypeScriptConfig::default(),
            python: PythonConfig::default(),
            create_dirs: default_true(),
            override_files: default_true(),
        }
    }
}

/// Helper function to convert from TsImportStyle to generator's ImportStyle
impl From<TsImportStyle> for ImportStyle {
    fn from(style: TsImportStyle) -> Self {
        match style {
            TsImportStyle::Default => ImportStyle::Default,
            TsImportStyle::Named => ImportStyle::Named,
            TsImportStyle::Namespace => ImportStyle::Namespace,
        }
    }
}

// Default helper functions
fn default_true() -> bool {
    true
}

fn default_ts_extension() -> String {
    ".ts".to_string()
}

fn default_py_extension() -> String {
    ".py".to_string()
}

impl Config {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        
        Ok(())
    }
    
    /// Get the path to the output directory
    pub fn output_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.output_dir)
    }
    
    /// Get file extension for current target
    pub fn file_extension(&self) -> &str {
        match self.target {
            TargetLanguage::TypeScript => &self.typescript.file_extension,
            TargetLanguage::Python => &self.python.file_extension,
        }
    }
}

/// Error type for configuration operations
#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Parse(serde_json::Error),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::Parse(err)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "I/O error: {}", err),
            ConfigError::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(err) => Some(err),
            ConfigError::Parse(err) => Some(err),
        }
    }
}
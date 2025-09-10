//! Module for Zod schema generation from Protocol Buffer definitions
//!
//! This module contains structures and functions for parsing Zod annotations
//! from Protocol Buffer comments and generating Zod schemas.

pub mod metadata;
pub mod parser;
pub mod generator;
pub mod writer;
pub mod config;

#[cfg(test)]
mod tests;

pub use metadata::ZodMetadata;
pub use generator::{ZodGenerator, ZodGeneratorConfig, ImportStyle};
pub use writer::TypeScriptWriter;
pub use config::{Config, TargetLanguage, TsImportStyle};
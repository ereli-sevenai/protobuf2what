//! Buf plugin implementation
//!
//! This module contains the implementation of the Buf plugin protocol
//! for integrating with the Buf ecosystem.

use std::io::{self, Read, Write};
use std::path::PathBuf;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

use crate::parser::parse_proto_file;
use crate::zod::{
    ZodGenerator, ZodGeneratorConfig, ImportStyle, TypeScriptWriter,
    config::Config,
    parser::ZodAnnotationParser,
};

/// Plugin request from Buf
#[derive(Debug, Deserialize)]
struct PluginRequest {
    files: Vec<File>,
}

/// Proto file in a Buf plugin request
#[derive(Debug, Deserialize)]
struct File {
    name: String,
    content: String,
}

/// Plugin response to Buf
#[derive(Debug, Serialize)]
struct PluginResponse {
    files: Vec<ResponseFile>,
}

/// Generated file in a Buf plugin response
#[derive(Debug, Serialize)]
struct ResponseFile {
    name: String,
    content: String,
}

/// Run the plugin in Buf plugin mode
pub fn run_plugin() -> Result<(), io::Error> {
    info!("Running as a Buf plugin");
    
    // Read plugin request from stdin
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;
    
    // Parse the plugin request
    let request: PluginRequest = serde_json::from_slice(&buffer)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    info!("Received {} files from Buf", request.files.len());
    
    // Process each file
    let mut response_files = Vec::new();
    for file in &request.files {
        info!("Processing file: {}", file.name);
        
        // Parse the proto file
        let proto_file = match parse_proto_file(&file.content) {
            Ok(proto_file) => proto_file,
            Err(e) => {
                error!("Failed to parse file {}: {}", file.name, e);
                continue;
            }
        };
        
        // Extract Zod annotations
        let zod_metadata = ZodAnnotationParser::parse_file(&proto_file, &file.content);
        
        // Create generator config
        let generator_config = ZodGeneratorConfig {
            import_style: ImportStyle::Named,
            single_file: true,
            output_dir: "generated".to_string(),
        };
        
        // Generate Zod schemas
        let generator = ZodGenerator::new(zod_metadata, generator_config);
        let generated_files = generator.generate(&proto_file);
        
        // Add generated files to response
        for (filename, content) in generated_files {
            let output_name = format!("{}.ts", file.name.replace(".proto", ""));
            response_files.push(ResponseFile {
                name: output_name,
                content,
            });
        }
    }
    
    // Create and send response
    let response = PluginResponse {
        files: response_files,
    };
    
    let response_json = serde_json::to_string(&response)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    io::stdout().write_all(response_json.as_bytes())?;
    
    info!("Buf plugin completed successfully");
    
    Ok(())
}

/// Check if the program is running as a Buf plugin
pub fn is_plugin_mode() -> bool {
    // Check if a special environment variable is set
    std::env::var("BUF_PLUGIN_MODE").is_ok() || 
        // Or if we have input on stdin and no arguments
        (!atty::is(atty::Stream::Stdin) && std::env::args().len() <= 1)
}
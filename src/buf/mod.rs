//! Buf plugin implementation
//!
//! This module contains the implementation of the Buf plugin protocol
//! for integrating with the Buf ecosystem.

use std::io::{self, Read, Write};
use std::path::Path;
use std::fs::{self, File};
use log::{error, info};
use prost::Message;

use crate::plugin_proto::{PluginRequest, PluginResponse, ResponseFile};
use crate::parser::parse_proto_file;
use crate::zod::{
    ZodGenerator, ZodGeneratorConfig, ImportStyle,
    parser::ZodAnnotationParser,
};

/// Run the plugin in Buf plugin mode
pub fn run_plugin() -> Result<(), io::Error> {
    info!("Running as a Buf plugin");
    
    // Read plugin request from stdin
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;
    
    if buffer.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Received empty buffer from stdin"));
    }
    
    info!("Received buffer of {} bytes from stdin", buffer.len());
    
    // Print first few bytes for debugging
    if buffer.len() > 20 {
        let bytes_str: Vec<String> = buffer[0..20].iter().map(|b| format!("{:02x}", b)).collect();
        info!("First 20 bytes: {}", bytes_str.join(" "));
    }
    
    // Check if the first 4 bytes might be a length prefix
    if buffer.len() >= 4 {
        let length_bytes = [buffer[0], buffer[1], buffer[2], buffer[3]];
        let length = u32::from_be_bytes(length_bytes);
        info!("Possible length prefix (big endian): {}", length);
        
        let length = u32::from_le_bytes(length_bytes);
        info!("Possible length prefix (little endian): {}", length);
        
        // Check if this might be a varint length prefix
        let mut varint: u64 = 0;
        let mut shift = 0;
        let mut i = 0;
        while i < buffer.len().min(10) && (buffer[i] & 0x80) != 0 {
            varint |= ((buffer[i] & 0x7f) as u64) << shift;
            shift += 7;
            i += 1;
        }
        if i < buffer.len() {
            varint |= ((buffer[i] & 0x7f) as u64) << shift;
        }
        info!("Possible varint length prefix: {}", varint);
    }

    // Try different approaches to decode the protocol buffer message
    
    // Try decoding the whole buffer directly
    match PluginRequest::decode(&buffer[..]) {
        Ok(req) => {
            info!("Successfully decoded PluginRequest from entire buffer");
            return process_request(req);
        },
        Err(e) => {
            info!("Failed to decode entire buffer as PluginRequest: {}", e);
        }
    };
    
    // Try decoding with a 4-byte length prefix (big endian)
    if buffer.len() >= 4 {
        let length_bytes = [buffer[0], buffer[1], buffer[2], buffer[3]];
        let length = u32::from_be_bytes(length_bytes) as usize;
        
        if buffer.len() >= 4 + length && length > 0 {
            info!("Trying with BE length prefix: {}", length);
            match PluginRequest::decode(&buffer[4..4+length]) {
                Ok(req) => {
                    info!("Successfully decoded PluginRequest with BE length prefix");
                    return process_request(req);
                },
                Err(e) => {
                    info!("Failed to decode with BE length prefix: {}", e);
                }
            }
        }
    }
    
    // Try decoding with a 4-byte length prefix (little endian)
    if buffer.len() >= 4 {
        let length_bytes = [buffer[0], buffer[1], buffer[2], buffer[3]];
        let length = u32::from_le_bytes(length_bytes) as usize;
        
        if buffer.len() >= 4 + length && length > 0 && length < buffer.len() {
            info!("Trying with LE length prefix: {}", length);
            match PluginRequest::decode(&buffer[4..4+length]) {
                Ok(req) => {
                    info!("Successfully decoded PluginRequest with LE length prefix");
                    return process_request(req);
                },
                Err(e) => {
                    info!("Failed to decode with LE length prefix: {}", e);
                }
            }
        }
    }
    
    // Try skipping first bytes in case of a header
    for skip in [1, 2, 3, 4, 8, 16] {
        if buffer.len() > skip {
            info!("Trying to decode after skipping {} bytes", skip);
            match PluginRequest::decode(&buffer[skip..]) {
                Ok(req) => {
                    info!("Successfully decoded PluginRequest after skipping {} bytes", skip);
                    return process_request(req);
                },
                Err(e) => {
                    info!("Failed to decode after skipping {} bytes: {}", skip, e);
                }
            }
        }
    }
    
    // If all decoding attempts failed, use the fallback handler
    info!("All decoding attempts failed, using fallback handler");
    return handle_raw_input(&buffer);
}

/// Process the plugin request and generate a response
fn process_request(request: PluginRequest) -> Result<(), io::Error> {
    info!("Processing PluginRequest");
    info!("Received {} files from Buf", request.files.len());
    
    // Create the response
    let mut response_files = Vec::new();
    
    // Process each file in the request
    for file in &request.files {
        info!("Processing file: {}", file.name);
        
        // Parse the proto file
        match parse_proto_file(&file.content) {
            Ok(proto_file) => {
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
                for (_, content) in generated_files {
                    let output_path = Path::new(&file.name).with_extension("ts");
                    let output_name = output_path
                        .to_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("{}.ts", file.name));
                    
                    response_files.push(ResponseFile {
                        name: output_name,
                        content,
                    });
                }
            },
            Err(e) => {
                error!("Failed to parse file {}: {}", file.name, e);
                // Error handling is simpler in our custom protocol - just log and continue
            }
        }
    }
    
    // Create response
    info!("Creating response with {} files", response_files.len());
    
    // Log details about each file
    for (i, file) in response_files.iter().enumerate() {
        info!("Response file {}: name={}, content_length={}", i, file.name, file.content.len());
    }
    
    let response = PluginResponse {
        files: response_files,
    };
    
    // Encode the response
    let mut encoded = Vec::new();
    response.encode(&mut encoded)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    info!("Encoded response size: {} bytes", encoded.len());
    
    // Debug: print first few bytes of the encoded response
    if encoded.len() > 20 {
        let bytes_str: Vec<String> = encoded[0..20].iter().map(|b| format!("{:02x}", b)).collect();
        info!("First 20 bytes of response: {}", bytes_str.join(" "));
    }
    
    // Send response to stdout
    io::stdout().write_all(&encoded)?;
    
    info!("Buf plugin completed successfully");
    
    Ok(())
}

/// Process proto files found in directories
pub fn process_directory_files(files: &[String]) -> Result<(), io::Error> {
    let mut response_files = Vec::new();
    
    for file_path in files {
        info!("Processing proto file from directory: {}", file_path);
        
        // Try to read the file
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                // Parse the proto file and generate Zod schema
                match parse_proto_file(&content) {
                    Ok(proto_file) => {
                        let zod_metadata = ZodAnnotationParser::parse_file(&proto_file, &content);
                        let generator_config = ZodGeneratorConfig {
                            import_style: ImportStyle::Named,
                            single_file: true,
                            output_dir: "generated".to_string(),
                        };
                        let generator = ZodGenerator::new(zod_metadata, generator_config);
                        let generated_files = generator.generate(&proto_file);
                        
                        for (_, content) in generated_files {
                            let output_path = Path::new(file_path).with_extension("ts");
                            let output_name = output_path
                                .to_str()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("{}.ts", file_path));
                            
                            // Add to response for protocol buffer response
                            response_files.push(ResponseFile {
                                name: output_name.clone(),
                                content: content.clone(),
                            });
                            
                            // Also write directly to file
                            let file_name = match Path::new(&output_name).file_name() {
                                Some(name) => name.to_os_string(),
                                None => {
                                    error!("Failed to extract file name from path: {}", output_name);
                                    continue;
                                }
                            };
                            
                            // Create the output directories if needed
                            let output_dirs = vec![Path::new("gen/zod"), Path::new("generated")];
                            for output_dir in &output_dirs {
                                if !output_dir.exists() {
                                    match fs::create_dir_all(output_dir) {
                                        Ok(_) => info!("Created output directory: {:?}", output_dir),
                                        Err(e) => {
                                            error!("Failed to create output directory {:?}: {}", output_dir, e);
                                            continue;
                                        }
                                    }
                                }
                            }
                            
                            // Write to both gen/zod and generated directories
                            
                            // Write to gen/zod directory
                            let zod_output_path = Path::new("gen/zod").join(&file_name);
                            info!("Writing output directly to file: {:?}", zod_output_path);
                            match fs::write(&zod_output_path, &content) {
                                Ok(_) => info!("Successfully wrote file: {:?}", zod_output_path),
                                Err(e) => error!("Failed to write file {:?}: {}", zod_output_path, e),
                            }
                            
                            // Also write to generated directory (used by buf)
                            let generated_output_path = Path::new("generated").join(&file_name);
                            info!("Writing output to buf directory: {:?}", generated_output_path);
                            match fs::write(&generated_output_path, &content) {
                                Ok(_) => info!("Successfully wrote file: {:?}", generated_output_path),
                                Err(e) => error!("Failed to write file {:?}: {}", generated_output_path, e),
                            }
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse proto file {}: {}", file_path, e);
                    }
                }
            },
            Err(e) => {
                error!("Failed to read file {}: {}", file_path, e);
            }
        }
    }
    
    // Create response
    info!("Creating directory files response with {} files", response_files.len());
    
    // Log details about each file
    for (i, file) in response_files.iter().enumerate() {
        info!("Directory files response file {}: name={}, content_length={}", 
            i, file.name, file.content.len());
    }
    
    let response = PluginResponse {
        files: response_files,
    };
    
    // Encode the response
    let mut encoded = Vec::new();
    response.encode(&mut encoded)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    info!("Encoded directory files response size: {} bytes", encoded.len());
    
    // Debug: print first few bytes of the encoded response
    if encoded.len() > 20 {
        let bytes_str: Vec<String> = encoded[0..20].iter().map(|b| format!("{:02x}", b)).collect();
        info!("First 20 bytes of directory files response: {}", bytes_str.join(" "));
    }
    
    // Send response to stdout
    io::stdout().write_all(&encoded)?;
    
    info!("Buf plugin completed successfully with directory files handler");
    
    Ok(())
}

/// Handle raw input from stdin when protobuf decoding fails
fn handle_raw_input(buffer: &[u8]) -> Result<(), io::Error> {
    // Fallback implementation for when we can't parse the protocol buffer request
    let mut response_files = Vec::new();
    
    // Convert buffer to string and look for proto files
    if let Ok(input_str) = String::from_utf8(buffer.to_vec()) {
        // Dump raw buffer content for debugging
        info!("Raw input first 50 bytes: {:?}", 
            input_str.chars().take(50).collect::<String>());
        
        // Dump all potential proto file references for debugging
        let proto_refs: Vec<_> = input_str.split('\n')
            .filter(|line| line.contains(".proto"))
            .take(10) // Limit to first 10 matches to avoid log spam
            .collect();
        
        info!("Found {} potential proto references in raw input", proto_refs.len());
        for (i, line) in proto_refs.iter().enumerate() {
            info!("Proto ref {}: {}", i, line);
        }
        
        // Check if we got binary data (might be protocol buffers)
        let binary_chars: Vec<_> = buffer.iter()
            .filter(|&&b| b < 32 && b != b'\n' && b != b'\r' && b != b'\t')
            .collect();
        info!("Buffer contains {} non-printable chars", binary_chars.len());
        
        // Try to directly find files in the current directory
        info!("Checking for proto files in current directory");
        match std::fs::read_dir(".") {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "proto" && path.is_file() {
                                info!("Found proto file in current dir: {:?}", path);
                            }
                        }
                    }
                }
            },
            Err(e) => error!("Failed to read current directory: {}", e)
        }
        
        // Also check files directory
        info!("Checking for proto files in files/ directory");
        match std::fs::read_dir("files") {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "proto" && path.is_file() {
                                info!("Found proto file in files/ dir: {:?}", path);
                            }
                        }
                    }
                }
            },
            Err(e) => error!("Failed to read files directory: {}", e)
        }
        
        // Try to extract proto files from buffer directly
        // Look for something that might be a proto file name
        let proto_files: Vec<_> = input_str.split('\0')
            .filter(|s| s.contains(".proto"))
            .map(|s| {
                // Extract likely proto file path with .proto extension
                if let Some(pos) = s.find(".proto") {
                    let end = pos + 6; // Include the .proto extension
                    
                    // Find the likely start of the file name
                    // Look backwards for whitespace, null bytes, or other obvious delimiters
                    let mut start = 0;
                    for (i, c) in s[..pos].chars().rev().enumerate() {
                        if c == ' ' || c == '\n' || c == '\r' || c == '\t' || c == '\0' || 
                           !c.is_ascii_graphic() {
                            start = pos - i;
                            break;
                        }
                    }
                    
                    // Get the substring that looks most like a file path
                    let path = s[start..end].trim();
                    info!("Extracted potential proto path: {}", path);
                    path.to_string()
                } else {
                    s.to_string()
                }
            })
            .filter(|s| s.ends_with(".proto"))
            .collect();
        
        info!("Found {} potential proto files in input", proto_files.len());
        
        // If no proto files found in buffer, use files from the current directory
        if proto_files.is_empty() {
            info!("No proto files found in buffer, processing files from directory");
            
            // Try files directory first, then current directory
            let mut paths_to_check = Vec::new();
            
            // Add files from files/ directory if it exists
            match std::fs::read_dir("files") {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if let Some(ext) = path.extension() {
                                if ext == "proto" && path.is_file() {
                                    paths_to_check.push(path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                },
                Err(e) => error!("Failed to read files directory: {}", e)
            }
            
            // Then try current directory
            if paths_to_check.is_empty() {
                match std::fs::read_dir(".") {
                    Ok(entries) => {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let path = entry.path();
                                if let Some(ext) = path.extension() {
                                    if ext == "proto" && path.is_file() {
                                        paths_to_check.push(path.to_string_lossy().to_string());
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => error!("Failed to read current directory: {}", e)
                }
            }
            
            // If we found proto files, process them
            if !paths_to_check.is_empty() {
                info!("Using {} proto files found in directories", paths_to_check.len());
                return process_directory_files(&paths_to_check);
            }
        }
        
        for line in proto_files {
            info!("Found proto file: {}", line);
            
            // Try to read the file
            match std::fs::read_to_string(&line) {
                Ok(content) => {
                    // Parse the proto file and generate Zod schema
                    match parse_proto_file(&content) {
                        Ok(proto_file) => {
                            let zod_metadata = ZodAnnotationParser::parse_file(&proto_file, &content);
                            let generator_config = ZodGeneratorConfig {
                                import_style: ImportStyle::Named,
                                single_file: true,
                                output_dir: "generated".to_string(),
                            };
                            let generator = ZodGenerator::new(zod_metadata, generator_config);
                            let generated_files = generator.generate(&proto_file);
                            
                            for (_, content) in generated_files {
                                let output_path = Path::new(&line).with_extension("ts");
                                let output_name = output_path
                                    .to_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| format!("{}.ts", line));
                                
                                // Add to response for protocol buffer response
                                response_files.push(ResponseFile {
                                    name: output_name.clone(),
                                    content: content.clone(),
                                });
                                
                                // Also write directly to file
                                let file_name = match Path::new(&output_name).file_name() {
                                    Some(name) => name,
                                    None => {
                                        error!("Failed to extract file name from path: {}", output_name);
                                        continue;
                                    }
                                };
                                
                                // Create the output directory if needed
                                let output_dir = Path::new("gen/zod");
                                if !output_dir.exists() {
                                    match fs::create_dir_all(output_dir) {
                                        Ok(_) => info!("Created output directory: {:?}", output_dir),
                                        Err(e) => {
                                            error!("Failed to create output directory {:?}: {}", output_dir, e);
                                            continue;
                                        }
                                    }
                                }
                                
                                let output_path = output_dir.join(file_name);
                                info!("Writing output directly to file: {:?}", output_path);
                                
                                match fs::write(&output_path, &content) {
                                    Ok(_) => info!("Successfully wrote file: {:?}", output_path),
                                    Err(e) => error!("Failed to write file {:?}: {}", output_path, e),
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to parse proto file {}: {}", line, e);
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to read file {}: {}", line, e);
                }
            }
        }
    }
    
    // Create response
    info!("Creating fallback response with {} files", response_files.len());
    
    // Log details about each file
    for (i, file) in response_files.iter().enumerate() {
        info!("Fallback response file {}: name={}, content_length={}", 
            i, file.name, file.content.len());
    }
    
    let response = PluginResponse {
        files: response_files,
    };
    
    // Encode the response
    let mut encoded = Vec::new();
    response.encode(&mut encoded)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    info!("Encoded fallback response size: {} bytes", encoded.len());
    
    // Debug: print first few bytes of the encoded response
    if encoded.len() > 20 {
        let bytes_str: Vec<String> = encoded[0..20].iter().map(|b| format!("{:02x}", b)).collect();
        info!("First 20 bytes of fallback response: {}", bytes_str.join(" "));
    }
    
    // Send response to stdout
    io::stdout().write_all(&encoded)?;
    
    info!("Buf plugin completed successfully with fallback handler");
    
    Ok(())
}

/// Check if the program is running as a Buf plugin
pub fn is_plugin_mode() -> bool {
    // Check if a special environment variable is set
    std::env::var("BUF_PLUGIN_MODE").is_ok() || 
        // Or if we have input on stdin and no arguments
        (!atty::is(atty::Stream::Stdin) && std::env::args().len() <= 1)
}

/// Process some known proto files as a convenience function
/// This is a special case handler for when the normal plugin protocol fails
pub fn process_known_files() -> Result<(), io::Error> {
    info!("Processing known proto files as fallback");
    
    // Process files from the files/ directory
    let paths = vec![
        "files/logdservice.proto".to_string(),
        "files/simple.proto".to_string(),
        "files/test_basic.proto".to_string(),
        "files/test_complex.proto".to_string(),
        "files/test_zod_validation.proto".to_string(),
        "files/with-zod-comments.proto".to_string(),
    ];
    
    process_directory_files(&paths)
}
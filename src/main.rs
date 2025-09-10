use log::{error, info};
use protobuf_to_zod::parser::parse_proto_file;
use protobuf_to_zod::zod::{
    Config, ImportStyle, TargetLanguage, TsImportStyle,
    ZodGenerator, ZodGeneratorConfig, TypeScriptWriter,
    parser::ZodAnnotationParser,
};
use protobuf_to_zod::buf;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use clap::{App, Arg, SubCommand};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    // Check if running as a Buf plugin
    if buf::is_plugin_mode() {
        return buf::run_plugin().map_err(|e| e.into());
    }
    
    // Parse command line arguments
    let matches = App::new("Protobuf to Zod")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Protobuf to Zod Contributors")
        .about("Convert Protocol Buffer definitions to Zod schemas")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .arg(Arg::with_name("input")
            .short("i")
            .long("input")
            .value_name("FILE")
            .help("Input proto file")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("output-dir")
            .short("o")
            .long("output-dir")
            .value_name("DIRECTORY")
            .help("Output directory for generated files")
            .takes_value(true))
        .arg(Arg::with_name("typescript")
            .long("typescript")
            .help("Generate TypeScript/Zod schemas"))
        .arg(Arg::with_name("python")
            .long("python")
            .help("Generate Python/Pydantic schemas"))
        .arg(Arg::with_name("import-style")
            .long("import-style")
            .value_name("STYLE")
            .help("Import style for Zod (default, named, namespace)")
            .takes_value(true))
        .get_matches();
    
    // Load or create configuration
    let config = if let Some(config_path) = matches.value_of("config") {
        Config::from_file(config_path)?
    } else {
        let mut config = Config::default();
        
        // Override configuration with command line arguments
        if let Some(output_dir) = matches.value_of("output-dir") {
            config.output_dir = output_dir.to_string();
        }
        
        if matches.is_present("typescript") {
            config.target = TargetLanguage::TypeScript;
        } else if matches.is_present("python") {
            config.target = TargetLanguage::Python;
        }
        
        if let Some(import_style) = matches.value_of("import-style") {
            config.typescript.import_style = match import_style {
                "default" => TsImportStyle::Default,
                "named" => TsImportStyle::Named,
                "namespace" => TsImportStyle::Namespace,
                _ => {
                    return Err(format!("Invalid import style: {}", import_style).into());
                }
            };
        }
        
        config
    };
    
    // Get input file
    let input_file = matches.value_of("input").unwrap();
    let proto_path = Path::new(input_file);
    
    if !proto_path.exists() {
        return Err(format!("Input file does not exist: {}", proto_path.display()).into());
    }
    
    info!("Reading Protobuf file from: {}", proto_path.display());
    
    let proto_content = fs::read_to_string(&proto_path).map_err(|e| {
        error!("Failed to read the proto file: {}", e);
        format!(
            "Failed to read the proto file '{}': {}",
            proto_path.display(),
            e
        )
    })?;
    
    info!("Parsing Protobuf file content");
    
    let proto_file = parse_proto_file(&proto_content).map_err(|e| {
        error!("Failed to parse Protobuf file: {}", e);
        format!("Failed to parse Protobuf file: {}", e)
    })?;
    
    info!("Successfully parsed Protobuf file");
    
    // Extract Zod annotations from comments
    let zod_metadata = ZodAnnotationParser::parse_file(&proto_file, &proto_content);
    
    // Generate schemas based on target language
    match config.target {
        TargetLanguage::TypeScript => {
            // Create generator config
            let generator_config = ZodGeneratorConfig {
                import_style: config.typescript.import_style.into(),
                single_file: config.typescript.single_file,
                output_dir: config.output_dir.clone(),
            };
            
            // Create generator
            let generator = ZodGenerator::new(zod_metadata, generator_config);
            
            // Generate schemas
            let generated_files = generator.generate(&proto_file);
            
            // Create output directory if it doesn't exist
            if config.create_dirs {
                fs::create_dir_all(&config.output_dir)?;
            }
            
            // Write schemas to files
            let writer = TypeScriptWriter::new(&config.output_dir, config.create_dirs);
            
            for (filename, content) in generated_files {
                writer.write_file(&filename, &content)?;
            }
            
            info!("Successfully generated TypeScript/Zod schemas in {}", config.output_dir);
        },
        TargetLanguage::Python => {
            // Python/Pydantic generation not yet implemented
            return Err("Python/Pydantic generation not yet implemented".into());
        },
    }
    
    Ok(())
}
use std::io::Result;
use prost_build;

fn main() -> Result<()> {
    // Build protos for the Buf plugin protocol (using the official protoc plugin definition)
    println!("cargo:rerun-if-changed=proto/buf/plugin.proto");
    
    // Configure prost_build
    let mut config = prost_build::Config::new();
    config.btree_map(["."]);
    config.compile_protos(&["proto/buf/plugin.proto"], &["proto/"])?;
    
    Ok(())
}
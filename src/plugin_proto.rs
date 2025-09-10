// Include the generated Protocol Buffer code
pub mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

// Re-export the types for convenience
// The prost-generated names are PluginRequest and PluginResponse
pub use plugin::{PluginRequest, PluginResponse, File, ResponseFile};
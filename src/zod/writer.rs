use log::{debug, info};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Writer for generated TypeScript files containing Zod schemas
pub struct TypeScriptWriter {
    /// Base directory for output files
    output_dir: PathBuf,
    
    /// Whether to create directories if they don't exist
    create_dirs: bool,
}

/// Result type for writer operations
pub type WriterResult<T> = Result<T, io::Error>;

impl TypeScriptWriter {
    /// Create a new TypeScriptWriter with the given output directory
    pub fn new<P: AsRef<Path>>(output_dir: P, create_dirs: bool) -> Self {
        TypeScriptWriter {
            output_dir: output_dir.as_ref().to_path_buf(),
            create_dirs,
        }
    }
    
    /// Write a TypeScript file with the given filename and content
    pub fn write_file(&self, filename: &str, content: &str) -> WriterResult<()> {
        let file_path = self.output_dir.join(filename);
        
        // Create the parent directory if it doesn't exist and create_dirs is true
        if self.create_dirs {
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // Write the file
        debug!("Writing file: {}", file_path.display());
        fs::write(&file_path, content)?;
        info!("Successfully wrote file: {}", file_path.display());
        
        Ok(())
    }
    
    /// Write multiple TypeScript files with the given filenames and contents
    pub fn write_files(&self, files: &[(String, String)]) -> WriterResult<()> {
        for (filename, content) in files {
            self.write_file(filename, content)?;
        }
        
        Ok(())
    }
    
    /// Get the full path to a file in the output directory
    pub fn get_file_path(&self, filename: &str) -> PathBuf {
        self.output_dir.join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_write_file() {
        let temp_dir = tempdir().unwrap();
        let writer = TypeScriptWriter::new(temp_dir.path(), true);
        
        let filename = "test.ts";
        let content = "export const test = 'test';";
        
        writer.write_file(filename, content).unwrap();
        
        let file_path = temp_dir.path().join(filename);
        let read_content = fs::read_to_string(file_path).unwrap();
        
        assert_eq!(read_content, content);
    }
    
    #[test]
    fn test_write_files() {
        let temp_dir = tempdir().unwrap();
        let writer = TypeScriptWriter::new(temp_dir.path(), true);
        
        let files = vec![
            ("test1.ts".to_string(), "export const test1 = 'test1';".to_string()),
            ("test2.ts".to_string(), "export const test2 = 'test2';".to_string()),
        ];
        
        writer.write_files(&files).unwrap();
        
        for (filename, expected_content) in files {
            let file_path = temp_dir.path().join(filename);
            let read_content = fs::read_to_string(file_path).unwrap();
            
            assert_eq!(read_content, expected_content);
        }
    }
    
    #[test]
    fn test_write_file_with_nested_directories() {
        let temp_dir = tempdir().unwrap();
        let writer = TypeScriptWriter::new(temp_dir.path(), true);
        
        let filename = "nested/dir/test.ts";
        let content = "export const test = 'test';";
        
        writer.write_file(filename, content).unwrap();
        
        let file_path = temp_dir.path().join(filename);
        let read_content = fs::read_to_string(file_path).unwrap();
        
        assert_eq!(read_content, content);
    }
}
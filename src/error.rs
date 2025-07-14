use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Dictionary generation failed: {0}")]
    DictionaryError(String),
    
    #[error("Invalid file format")]
    InvalidFormat,
    
    #[error("Path traversal attack detected")]
    PathTraversal,
}

#[derive(Error, Debug)]
pub enum DecompressionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid file format")]
    InvalidFormat,
    
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
} 
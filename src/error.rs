use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Erreur de compression: {0}")]
    CompressionError(String),
    
    #[error("Erreur d'entrée/sortie: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Erreur de dictionnaire: {0}")]
    DictionaryError(String),
} 
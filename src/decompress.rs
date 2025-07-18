use std::fs::{self, File};
use std::io::{Read, Write, Cursor};
use std::path::PathBuf;
use anyhow::{Result, Context};
use tracing::info;
use zstd::decode_all;

use crate::error::DecompressionError;

pub struct DecompressionOptions {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

impl Default for DecompressionOptions {
    fn default() -> Self {
        Self {
            input_path: PathBuf::new(),
            output_path: PathBuf::new(),
        }
    }
}

fn sanitize_path(path: &str) -> Result<PathBuf> {
    // Validate and sanitize path to prevent path traversal attacks
    let path = path.trim();
    
    // Reject empty paths
    if path.is_empty() {
        return Err(DecompressionError::InvalidFormat.into());
    }
    
    // Reject absolute paths
    if path.starts_with('/') || path.starts_with('\\') || path.contains(':') {
        return Err(DecompressionError::InvalidFormat.into());
    }
    
    // Split path into components and validate each one
    let components: Vec<&str> = path.split(['/', '\\']).collect();
    let mut safe_components = Vec::new();
    
    for component in components {
        // Reject dangerous components
        if component == "." || component == ".." || component.is_empty() {
            continue; // Skip dangerous components
        }
        
        // Sanitize component by removing invalid characters
        let sanitized = component.replace(['<', '>', ':', '"', '|', '?', '*'], "_");
        if !sanitized.is_empty() {
            safe_components.push(sanitized);
        }
    }
    
    // Ensure we have at least one valid component
    if safe_components.is_empty() {
        return Err(DecompressionError::InvalidFormat.into());
    }
    
    // Build safe path
    let mut safe_path = PathBuf::new();
    for component in safe_components {
        safe_path.push(component);
    }
    
    Ok(safe_path)
}

pub fn decompress_archive(options: &DecompressionOptions) -> Result<()> {
    info!("Démarrage de la décompression de {:?}", options.input_path);
    
    let mut input_file = File::open(&options.input_path)
        .context("Impossible d'ouvrir le fichier d'entrée")?;

    // Créer le dossier de sortie s'il n'existe pas
    fs::create_dir_all(&options.output_path)?;
    println!("Dossier de sortie créé : {:?}", options.output_path);

    // Lire la taille du dictionnaire
    let mut dict_size_bytes = [0u8; 8];
    input_file.read_exact(&mut dict_size_bytes)?;
    let dict_size = u64::from_le_bytes(dict_size_bytes) as usize;
    
    // Validation: taille de dictionnaire raisonnable
    if dict_size > 100 * 1024 * 1024 { // 100MB max
        return Err(DecompressionError::InvalidFormat.into());
    }
    
    info!("Taille du dictionnaire: {} octets", dict_size);

    // Lire le dictionnaire
    let mut dict = vec![0u8; dict_size];
    input_file.read_exact(&mut dict)?;

    // Lire les données compressées
    let mut compressed_data = Vec::new();
    input_file.read_to_end(&mut compressed_data)?;
    info!("Données compressées lues: {} octets", compressed_data.len());

    // Décompresser les données
    let decompressed_data = decode_all(Cursor::new(&compressed_data))?;
    info!("Données décompressées: {} octets", decompressed_data.len());

    // Parcourir les données décompressées
    let mut cursor = Cursor::new(decompressed_data);
    loop {
        let offset = cursor.position();
        // Lire le chemin du fichier
        let mut path_bytes = Vec::new();
        let mut byte = [0u8; 1];
        while cursor.read_exact(&mut byte).is_ok() && byte[0] != 0 {
            path_bytes.push(byte[0]);
        }
        if path_bytes.is_empty() {
            println!("Fin de l'archive à l'offset {}", offset);
            break; // Fin du fichier
        }
        let path_str = String::from_utf8(path_bytes)
            .map_err(|_| DecompressionError::InvalidFormat)?;
        println!("Lecture du fichier : {} (offset: {})", path_str, offset);
        
        // Sanitize path to prevent path traversal attacks
        let sanitized_path = sanitize_path(&path_str)?;
        let file_path = options.output_path.join(&sanitized_path);
        
        // Additional security check: ensure the final path is within output directory
        let canonical_output = options.output_path.canonicalize()
            .context("Failed to canonicalize output path")?;
        if let Ok(canonical_file) = file_path.canonicalize() {
            if !canonical_file.starts_with(&canonical_output) {
                return Err(DecompressionError::InvalidFormat.into());
            }
        }
        println!("Chemin complet : {:?}", file_path);

        // Créer les dossiers parents si nécessaire
        if let Some(parent) = file_path.parent() {
            println!("Création du dossier parent : {:?}", parent);
            fs::create_dir_all(parent)?;
        }

        // Lire la taille du fichier (8 octets)
        let mut size_bytes = [0u8; 8];
        cursor.read_exact(&mut size_bytes)?;
        let size = u64::from_le_bytes(size_bytes) as usize;
        println!("Taille des données : {} octets (offset: {})", size, cursor.position());

        // Lire les données
        let mut buffer = vec![0u8; size];
        cursor.read_exact(&mut buffer)?;
        println!("Lecture de {} octets pour {} (offset après lecture: {})", size, path_str, cursor.position());

        // Écrire le fichier
        let mut output_file = File::create(&file_path)?;
        output_file.write_all(&buffer)?;
        println!("Fichier décompressé avec succès : {:?}", file_path);
    }

    println!("Décompression terminée avec succès");
    Ok(())
}

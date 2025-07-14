use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use rayon::prelude::*;
use walkdir::WalkDir;
use log::{info, warn};
use std::io::Cursor;
use zstd::encode_all;
use std::io::Read;
use anyhow::{Result, Context};
use zstd::dict::from_samples;

use crate::profile::{detect_profile, CompressionProfile};

use crate::error::CompressionError;

#[derive(Debug)]
pub struct CompressionOptions {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub threads: usize,
    pub level: i32,
    pub solid: bool,
}

#[derive(Debug, Clone, Copy)]
enum FileType {
    Text,
    Binary,
    Json,
    Lua,
    Python,
    Other,
}

fn detect_file_type(path: &Path) -> FileType {
    if let Some(ext) = path.extension() {
        match ext.to_str().unwrap_or("").to_lowercase().as_str() {
            "txt" | "md" | "log" => FileType::Text,
            "json" => FileType::Json,
            "lua" => FileType::Lua,
            "py" => FileType::Python,
            "bin" | "exe" | "dll" | "so" | "dylib" => FileType::Binary,
            _ => FileType::Other,
        }
    } else {
        FileType::Other
    }
}

pub fn compress_folder(options: &CompressionOptions) -> Result<(), CompressionError> {
    let start_time = std::time::Instant::now();
    let mut total_size = 0;
    let mut compressed_size = 0;

    println!("Démarrage de la compression du dossier : {:?}", options.input_path);

    // Collecter les fichiers et construire les dictionnaires
    let mut dictionaries: HashMap<CompressionProfile, Vec<u8>> = HashMap::new();
    let mut files_to_compress = Vec::new();

    for entry in WalkDir::new(&options.input_path) {
        let entry = entry.map_err(|e| CompressionError::Io(std::io::Error::other(e)))?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let relative_path = path.strip_prefix(&options.input_path)
                .map_err(|e| CompressionError::Io(std::io::Error::other(e)))?;
            println!("Fichier trouvé : {:?} (chemin relatif : {:?})", path, relative_path);
            
            let profile = detect_profile(path);
            let file_size = fs::metadata(path)?.len();

            // Ajouter les petits fichiers au dictionnaire
            if file_size < 1024 * 1024 { // 1MB
                let content = fs::read(path)?;
                dictionaries.entry(profile)
                    .or_insert_with(Vec::new)
                    .extend(content);
            }

            files_to_compress.push((path.to_path_buf(), relative_path.to_path_buf(), profile));
            total_size += file_size;
        }
    }

    println!("Nombre de fichiers à compresser : {}", files_to_compress.len());

    let compression_dicts = Arc::new(dictionaries);
    let results: Vec<Result<(PathBuf, Vec<u8>), CompressionError>> = files_to_compress.par_iter()
        .map(|(path, relative_path, profile)| {
            println!("Compressing file: {path:?}");
            let dict = compression_dicts.get(profile);
            process_file(path, dict, profile.get_compression_level())
                .map(|data| (relative_path.clone(), data))
        })
        .collect();

    // Écrire les résultats
    let mut output = fs::File::create(&options.output_path)?;
    println!("Création de l'archive : {:?}", options.output_path);
    
    for result in results {
        match result {
            Ok((relative_path, data)) => {
                // Écrire le chemin relatif
                let path_str = relative_path.to_string_lossy();
                println!("Écriture du fichier : {}", path_str);
                output.write_all(path_str.as_bytes())?;
                output.write_all(&[0])?; // Séparateur nul

                // Écrire la taille des données compressées
                let size = data.len() as u64;
                println!("Taille des données compressées : {} octets", size);
                output.write_all(&size.to_le_bytes())?;

                // Écrire les données compressées
                output.write_all(&data)?;
                compressed_size += data.len() as u64;
            }
            Err(e) => warn!("Erreur lors de la compression: {}", e),
        }
    }

    let duration = start_time.elapsed();
    let ratio = (compressed_size as f64 / total_size as f64) * 100.0;
    println!("Compression terminée en {:.2?}", duration);
    println!("Taille originale: {} octets", total_size);
    println!("Taille compressée: {} octets", compressed_size);
    println!("Ratio de compression: {:.2}%", ratio);

    Ok(())
}

fn process_file(
    path: &Path,
    _dict: Option<&Vec<u8>>,
    level: i32,
) -> Result<Vec<u8>, CompressionError> {
    let content = fs::read(path).map_err(CompressionError::Io)?;
    let file_type = detect_file_type(path);
    let processed_content = match file_type {
        FileType::Text | FileType::Json | FileType::Lua | FileType::Python => {
            // Prétraitement pour les fichiers texte
            let text = String::from_utf8_lossy(&content);
            let processed = text.lines()
                .map(|line| line.trim_end())
                .collect::<Vec<&str>>()
                .join("\n");
            processed.into_bytes()
        },
        FileType::Binary => {
            // Pas de prétraitement pour les fichiers binaires
            content
        },
        FileType::Other => content,
    };
    let compressed = encode_all(Cursor::new(processed_content), level)
        .map_err(|e| CompressionError::Io(std::io::Error::other(e)))?;
    Ok(compressed)
}

// Nouvelle fonction pour générer un dictionnaire global à partir de tous les fichiers
fn generate_global_dictionary(input_path: &Path) -> Result<Vec<u8>> {
    let mut samples = Vec::new();
    const MAX_SAMPLE_SIZE: usize = 64 * 1024; // 64 Ko par fichier
    const MAX_SAMPLES: usize = 100; // Limite stricte pour zstd

    for (i, entry) in fs::read_dir(input_path)?.enumerate() {
        if i >= MAX_SAMPLES { break; }
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let mut file = fs::File::open(&path)?;
            let mut buffer = vec![0u8; MAX_SAMPLE_SIZE];
            let bytes_read = file.read(&mut buffer)?;
            samples.push(buffer[..bytes_read].to_vec());
        }
    }

    if samples.len() < 8 {
        // Pas assez de fichiers pour générer un dictionnaire pertinent
        Ok(Vec::new())
    } else {
        let dict = from_samples(&samples, 64 * 1024)?; // 64KB de dictionnaire
        Ok(dict)
    }
}

pub fn compress_directory(options: &CompressionOptions) -> Result<()> {
    info!("Démarrage de la compression de {:?}", options.input_path);
    
    // Utiliser compress_folder avec gestion d'erreur appropriée
    if options.solid {
        // Mode solid : utiliser la compression simple
        compress_directory_solid(options)
    } else {
        // Mode normal : utiliser compress_folder
        compress_folder(options).map_err(|e| anyhow::anyhow!("Erreur de compression: {}", e))
    }
}

fn compress_directory_solid(options: &CompressionOptions) -> Result<()> {
    info!("Mode solid activé");
    
    // Générer le dictionnaire global
    let dict = generate_global_dictionary(&options.input_path)?;
    
    let output_file = fs::File::create(&options.output_path)
        .context("Impossible de créer le fichier de sortie")?;
    let mut writer = std::io::BufWriter::new(output_file);

    // Écrire la taille du dictionnaire
    writer.write_all(&(dict.len() as u64).to_le_bytes())?;
    // Écrire le dictionnaire
    writer.write_all(&dict)?;

    // Collecter tous les fichiers
    let mut all_data = Vec::new();
    let mut file_index = Vec::new();
    
    for entry in WalkDir::new(&options.input_path) {
        let entry = entry.map_err(|e| anyhow::anyhow!("Erreur walkdir: {}", e))?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let relative_path = path.strip_prefix(&options.input_path)
                .map_err(|e| anyhow::anyhow!("Erreur chemin: {}", e))?;
            
            let content = fs::read(&path)?;
            let start_offset = all_data.len();
            all_data.extend(content);
            let end_offset = all_data.len();
            
            file_index.push((relative_path.to_path_buf(), start_offset, end_offset));
        }
    }

    // Compression en mode solid avec le niveau et threads spécifiés
    info!("Compression avec niveau {} et {} threads", options.level, options.threads);
    let compressed = encode_all(Cursor::new(all_data), options.level)?;
    writer.write_all(&compressed)?;
    
    // Écrire l'index des fichiers
    writer.write_all(&(file_index.len() as u64).to_le_bytes())?;
    for (path, start, end) in file_index {
        let path_str = path.to_string_lossy();
        writer.write_all(&(path_str.len() as u64).to_le_bytes())?;
        writer.write_all(path_str.as_bytes())?;
        writer.write_all(&(start as u64).to_le_bytes())?;
        writer.write_all(&((end - start) as u64).to_le_bytes())?;
    }

    info!("Compression terminée avec succès");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_compression() {
        let temp_dir = tempdir().unwrap();
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&input_dir).unwrap();
        fs::create_dir(&output_dir).unwrap();

        // Créer des fichiers de test
        let text_content = "Ceci est un fichier texte de test avec beaucoup de répétitions. ".repeat(1000);
        create_test_file(&input_dir, "test.txt", text_content.as_bytes());

        let binary_content = vec![0u8; 1024 * 1024]; // 1MB de zéros
        create_test_file(&input_dir, "test.bin", &binary_content);

        let options = CompressionOptions {
            input_path: input_dir,
            output_path: output_dir.join("test.zpp"),
            threads: 2,
            level: 22,
            solid: false,
        };

        // Tester la compression
        compress_folder(&options).unwrap();

        // Vérifier que le fichier de sortie existe
        assert!(output_dir.join("test.zpp").exists());

        // Nettoyer
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_compression_profiles() {
        let temp_dir = tempdir().unwrap();
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&input_dir).unwrap();
        fs::create_dir(&output_dir).unwrap();

        // Créer des fichiers pour chaque profil
        let text_content = "Fichier texte de test".repeat(100);
        create_test_file(&input_dir, "text.txt", text_content.as_bytes());

        let binary_content = vec![0u8; 1024 * 10]; // 10KB de zéros
        create_test_file(&input_dir, "binary.bin", &binary_content);

        let image_content = vec![0u8; 1024 * 100]; // 100KB de données simulées d'image
        create_test_file(&input_dir, "image.jpg", &image_content);

        let unity_content = "Unity asset test data".repeat(100);
        create_test_file(&input_dir, "test.unity", unity_content.as_bytes());

        let options = CompressionOptions {
            input_path: input_dir,
            output_path: output_dir.join("test.zpp"),
            threads: 2,
            level: 22,
            solid: false,
        };

        // Tester la compression
        compress_folder(&options).unwrap();

        // Vérifier que le fichier de sortie existe
        assert!(output_dir.join("test.zpp").exists());

        // Nettoyer
        temp_dir.close().unwrap();
    }
}

/*!
 * ZippyPack - Système d'images avec déduplication
 * 
 * Créé par : Kamion (Matthéo Le Fur)
 * Date : 26/06/2025
 * Modifié le : 14/07/2025
 * 
 * Description : Implémentation du système d'images ZippyPack avec déduplication
 * par blocs de 64KB et compression zstd optimisée
 *
 * Version : 1.0.0
 */

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write, BufReader, BufWriter, Seek, SeekFrom};
use std::path::PathBuf;
use anyhow::Result;
use tracing::info;
use walkdir::WalkDir;
use zstd::{encode_all, decode_all};

const BLOCK_SIZE: usize = 65536; // 64KB blocks

#[derive(Debug, Clone)]
pub struct BlockHash([u8; 32]);

impl From<[u8; 32]> for BlockHash {
    fn from(hash: [u8; 32]) -> Self {
        BlockHash(hash)
    }
}

impl std::hash::Hash for BlockHash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for BlockHash {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for BlockHash {}

#[derive(Debug, Clone)]
pub struct DataBlock {
    pub compressed_data: Vec<u8>,
    pub original_size: usize,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
    pub is_directory: bool,
    pub blocks: Vec<BlockHash>,
}

#[derive(Debug)]
pub struct ImageHeader {
    pub version: u32,
    pub created: u64,
    pub total_files: u64,
    pub total_size: u64,
    pub compressed_size: u64,
    pub block_count: u64,
}

pub struct ImageOptions {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub compression_level: i32,
}

pub struct ExtractOptions {
    pub image_path: PathBuf,
    pub output_path: PathBuf,
}

fn calculate_hash(data: &[u8]) -> BlockHash {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Convert u64 to [u8; 32] (simple implementation)
    let mut result = [0u8; 32];
    result[0..8].copy_from_slice(&hash.to_le_bytes());
    BlockHash(result)
}

fn split_into_blocks(data: &[u8]) -> Vec<(BlockHash, Vec<u8>)> {
    data.chunks(BLOCK_SIZE)
        .map(|chunk| {
            let hash = calculate_hash(chunk);
            (hash, chunk.to_vec())
        })
        .collect()
}

pub fn create_image(options: &ImageOptions) -> Result<()> {
    info!("Création de l'image depuis {:?}", options.input_path);
    
    let mut file_entries = Vec::new();
    let mut block_store: HashMap<BlockHash, DataBlock> = HashMap::new();
    let mut total_size = 0u64;
    let mut total_files = 0u64;
    
    // Calcul du nombre total de fichiers pour la progression
    let total_entries: u64 = WalkDir::new(&options.input_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count() as u64;
    
    info!("Nombre total de fichiers à traiter: {}", total_entries);
    
    let start_time = std::time::Instant::now();
    let mut processed_size = 0u64;
    
    // Parcours récursif des fichiers
    for entry in WalkDir::new(&options.input_path) {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(&options.input_path)?;
        
        if path.is_dir() {
            file_entries.push(FileEntry {
                path: relative_path.to_path_buf(),
                size: 0,
                modified: 0,
                is_directory: true,
                blocks: Vec::new(),
            });
            continue;
        }
        
        let metadata = entry.metadata()?;
        let size = metadata.len();
        total_size += size;
        processed_size += size;
        total_files += 1;
        
        let modified = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        // Optimized reading with buffered I/O for large files
        use std::io::BufReader;
        
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        
        // For large files, read in chunks to avoid memory issues
        if size > 10 * 1024 * 1024 { // 10MB threshold
            const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
            let mut chunk = vec![0u8; CHUNK_SIZE];
            loop {
                let bytes_read = reader.read(&mut chunk)?;
                if bytes_read == 0 { break; }
                buffer.extend_from_slice(&chunk[..bytes_read]);
            }
        } else {
            reader.read_to_end(&mut buffer)?;
        }
        
        let blocks = split_into_blocks(&buffer);
        let mut file_blocks = Vec::new();
        
        for (hash, block_data) in blocks {
            file_blocks.push(hash.clone());
            
            // Déduplication : ne stocker que les blocs uniques
            if !block_store.contains_key(&hash) {
                let compressed = encode_all(&block_data[..], options.compression_level)?;
                block_store.insert(hash.clone(), DataBlock {
                    compressed_data: compressed,
                    original_size: block_data.len(),
                });
            }
        }
        
        file_entries.push(FileEntry {
            path: relative_path.to_path_buf(),
            size,
            modified,
            is_directory: false,
            blocks: file_blocks,
        });
        
        // Progression améliorée
        if total_files % 5 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let progress = (total_files as f64 / total_entries as f64) * 100.0;
            let speed_mbs = (processed_size as f64 / (1024.0 * 1024.0)) / elapsed;
            let eta_seconds = if speed_mbs > 0.0 {
                ((total_size - processed_size) as f64 / (1024.0 * 1024.0)) / speed_mbs
            } else {
                0.0
            };
            
            info!(
                "Progression: {:.1}% ({}/{}) - {:.1} MB/s - ETA: {:.0}s - Blocs uniques: {}",
                progress, total_files, total_entries, speed_mbs, eta_seconds, block_store.len()
            );
        }
    }
    
    // Calcul de la taille compressée
    let compressed_size: usize = block_store.values()
        .map(|block| block.compressed_data.len())
        .sum();
    
    // Écriture de l'image
    let mut output_file = BufWriter::new(File::create(&options.output_path)?);
    
    // Header
    let header = ImageHeader {
        version: 1,
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        total_files,
        total_size,
        compressed_size: compressed_size as u64,
        block_count: block_store.len() as u64,
    };
    
    // Sérialisation simple du header
    output_file.write_all(&header.version.to_le_bytes())?;
    output_file.write_all(&header.created.to_le_bytes())?;
    output_file.write_all(&header.total_files.to_le_bytes())?;
    output_file.write_all(&header.total_size.to_le_bytes())?;
    output_file.write_all(&header.compressed_size.to_le_bytes())?;
    output_file.write_all(&header.block_count.to_le_bytes())?;
    
    // Index des blocs
    for (hash, block) in &block_store {
        output_file.write_all(&hash.0)?; // 32 bytes hash
        output_file.write_all(&(block.original_size as u64).to_le_bytes())?;
        output_file.write_all(&(block.compressed_data.len() as u64).to_le_bytes())?;
    }
    
    // Données des blocs
    for block in block_store.values() {
        output_file.write_all(&block.compressed_data)?;
    }
    
    // Index des fichiers
    output_file.write_all(&(file_entries.len() as u64).to_le_bytes())?;
    for file_entry in &file_entries {
        let path_str = file_entry.path.to_string_lossy();
        let path_bytes = path_str.as_bytes();
        output_file.write_all(&(path_bytes.len() as u64).to_le_bytes())?;
        output_file.write_all(path_bytes)?;
        output_file.write_all(&file_entry.size.to_le_bytes())?;
        output_file.write_all(&file_entry.modified.to_le_bytes())?;
        output_file.write_all(&[if file_entry.is_directory { 1 } else { 0 }])?;
        output_file.write_all(&(file_entry.blocks.len() as u64).to_le_bytes())?;
        for block_hash in &file_entry.blocks {
            output_file.write_all(&block_hash.0)?;
        }
    }
    
    output_file.flush()?;
    
    let ratio = (compressed_size as f64 / total_size as f64) * 100.0;
    info!("Image créée: {} fichiers, {:.2}% de compression", total_files, 100.0 - ratio);
    info!("Taille originale: {} bytes", total_size);
    info!("Taille compressée: {} bytes", compressed_size);
    info!("Blocs uniques: {}", block_store.len());
    
    Ok(())
}

pub fn extract_image(options: &ExtractOptions) -> Result<()> {
    info!("Extraction de l'image {:?}", options.image_path);
    
    let mut input_file = BufReader::new(File::open(&options.image_path)?);
    
    // Lecture du header
    let mut buffer = [0u8; 8];
    input_file.read_exact(&mut buffer)?;
    let version = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    
    input_file.read_exact(&mut buffer)?;
    let _created = u64::from_le_bytes(buffer);
    
    input_file.read_exact(&mut buffer)?;
    let total_files = u64::from_le_bytes(buffer);
    
    input_file.read_exact(&mut buffer)?;
    let _total_size = u64::from_le_bytes(buffer);
    
    input_file.read_exact(&mut buffer)?;
    let _compressed_size = u64::from_le_bytes(buffer);
    
    input_file.read_exact(&mut buffer)?;
    let block_count = u64::from_le_bytes(buffer);
    
    info!("Version: {}, {} fichiers, {} blocs", version, total_files, block_count);
    
    // Lecture de l'index des blocs
    let mut block_index = HashMap::new();
    let mut current_offset = 6 * 8 + (block_count * (32 + 8 + 8)) as u64; // Skip to data section
    
    for _ in 0..block_count {
        let mut hash_bytes = [0u8; 32];
        input_file.read_exact(&mut hash_bytes)?;
        let hash = BlockHash(hash_bytes);
        
        input_file.read_exact(&mut buffer)?;
        let original_size = u64::from_le_bytes(buffer) as usize;
        
        input_file.read_exact(&mut buffer)?;
        let compressed_size = u64::from_le_bytes(buffer) as usize;
        
        block_index.insert(hash, (current_offset, original_size, compressed_size));
        current_offset += compressed_size as u64;
    }
    
    // Créer le dossier de sortie
    fs::create_dir_all(&options.output_path)?;
    
    // Lecture des métadonnées de fichiers
    input_file.read_exact(&mut buffer)?;
    let file_count = u64::from_le_bytes(buffer);
    
    for i in 0..file_count {
        // Lecture du chemin
        input_file.read_exact(&mut buffer)?;
        let path_len = u64::from_le_bytes(buffer) as usize;
        let mut path_bytes = vec![0u8; path_len];
        input_file.read_exact(&mut path_bytes)?;
        let relative_path = String::from_utf8(path_bytes)?;
        
        input_file.read_exact(&mut buffer)?;
        let _size = u64::from_le_bytes(buffer);
        
        input_file.read_exact(&mut buffer)?;
        let _modified = u64::from_le_bytes(buffer);
        
        let mut is_dir_byte = [0u8; 1];
        input_file.read_exact(&mut is_dir_byte)?;
        let is_directory = is_dir_byte[0] == 1;
        
        let full_path = options.output_path.join(&relative_path);
        
        if is_directory {
            fs::create_dir_all(&full_path)?;
            continue;
        }
        
        // Créer le dossier parent si nécessaire
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Lecture des blocs du fichier
        input_file.read_exact(&mut buffer)?;
        let block_count = u64::from_le_bytes(buffer);
        
        let mut file_data = Vec::new();
        for _ in 0..block_count {
            let mut hash_bytes = [0u8; 32];
            input_file.read_exact(&mut hash_bytes)?;
            let hash = BlockHash(hash_bytes);
            
            if let Some((offset, _original_size, compressed_size)) = block_index.get(&hash) {
                // Lecture du bloc compressé
                let mut file_handle = File::open(&options.image_path)?;
                file_handle.seek(SeekFrom::Start(*offset))?;
                let mut compressed_data = vec![0u8; *compressed_size];
                file_handle.read_exact(&mut compressed_data)?;
                
                // Décompression
                let decompressed = decode_all(&compressed_data[..])?;
                file_data.extend_from_slice(&decompressed);
            }
        }
        
        // Écriture du fichier
        let mut output_file = File::create(&full_path)?;
        output_file.write_all(&file_data)?;
        
        if (i + 1) % 100 == 0 {
            info!("Extrait {} fichiers", i + 1);
        }
    }
    
    info!("Extraction terminée: {} fichiers", file_count);
    Ok(())
}
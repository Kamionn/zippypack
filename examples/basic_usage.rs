/*!
 * ZippyPack - Exemple d'utilisation basique
 * 
 * Créé par : Kamion (Matthéo Le Fur)
 * Date : 28/06/2025
 * Modifié le : 14/07/2025
 * 
 * Description : Exemples d'utilisation de ZippyPack pour compression
 * et décompression de fichiers
 * 
 * Version : 1.0.0
 */

use std::path::PathBuf;
use zippy::compress::{compress_directory, CompressionOptions};
use zippy::decompress::{decompress_archive, DecompressionOptions};
use zippy::image::{create_image, extract_image, ImageOptions, ExtractOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("zippy=info")
        .init();

    // Exemple 1: Compression traditionnelle
    println!("=== Compression traditionnelle ===");
    let compress_options = CompressionOptions {
        input_path: PathBuf::from("./test_files"),
        output_path: PathBuf::from("./example.zpp"),
        threads: 4,
        level: 15,
        solid: true,
    };
    
    compress_directory(&compress_options)?;
    println!("Archive créée: example.zpp");

    // Exemple 2: Décompression
    println!("\n=== Décompression ===");
    let decompress_options = DecompressionOptions {
        input_path: PathBuf::from("./example.zpp"),
        output_path: PathBuf::from("./restored_files"),
    };
    
    decompress_archive(&decompress_options)?;
    println!("Fichiers restaurés dans: restored_files/");

    // Exemple 3: Système d'images avec déduplication
    println!("\n=== Système d'images ===");
    let image_options = ImageOptions {
        input_path: PathBuf::from("./test_files"),
        output_path: PathBuf::from("./example.zpak"),
        compression_level: 22,
    };
    
    create_image(&image_options)?;
    println!("Image créée: example.zpak");

    // Exemple 4: Extraction d'image
    println!("\n=== Extraction d'image ===");
    let extract_options = ExtractOptions {
        image_path: PathBuf::from("./example.zpak"),
        output_path: PathBuf::from("./extracted_files"),
    };
    
    extract_image(&extract_options)?;
    println!("Image extraite dans: extracted_files/");

    println!("\n✅ Tous les exemples terminés avec succès!");
    Ok(())
}
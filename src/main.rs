/*!
 * ZippyPack - Outil de compression moderne
 *
 * Créé par : Kamion (Matthéo Le Fur)
 * Date : 26/06/2025
 * Modifié le : 14/07/2025
 *
 * Description : Interface CLI principale pour ZippyPack, un outil de compression
 * avancé utilisant zstd avec déduplication par blocs et système d'images
 * 
 * Version : 1.0.0
 */

mod compress;
mod decompress;
mod profile;
mod image;

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;
use log::LevelFilter;
use compress::{compress_directory, CompressionOptions};
use decompress::{decompress_archive, DecompressionOptions};
use image::{create_image, extract_image, ImageOptions, ExtractOptions};

#[derive(Parser)]
#[command(name = "zippy")]
#[command(about = "Outil de compression .zpp rapide et intelligent", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Niveau de verbosité (0-4)
    #[arg(short, long, default_value = "2")]
    verbosity: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Compresser un dossier
    Compress {
        /// Dossier à compresser
        #[arg(short, long)]
        input: PathBuf,
        /// Fichier de sortie .zpp
        #[arg(short, long)]
        output: PathBuf,
        /// Nombre de threads à utiliser
        #[arg(short, long)]
        threads: Option<usize>,
        /// Niveau de compression (1-22)
        #[arg(short = 'l', long, default_value = "22")]
        level: i32,
        /// Mode solid (compresser en un seul flux)
        #[arg(long, default_value_t = false)]
        solid: bool,
    },
    /// Décompresser une archive .zpp
    Decompress {
        /// Archive .zpp à décompresser
        #[arg(short, long)]
        input: PathBuf,
        /// Dossier de sortie
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Créer une image système avec déduplication
    CreateImage {
        /// Dossier à capturer
        #[arg(short, long)]
        input: PathBuf,
        /// Fichier image .zpak
        #[arg(short, long)]
        output: PathBuf,
        /// Niveau de compression (1-22)
        #[arg(short = 'l', long, default_value = "22")]
        level: i32,
    },
    /// Extraire une image système
    ExtractImage {
        /// Fichier image .zpak
        #[arg(short, long)]
        input: PathBuf,
        /// Dossier de sortie
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configuration du logging
    let log_level = match cli.verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter_level(log_level)
        .init();

    match &cli.command {
        Commands::Compress { input, output, threads, level, solid } => {
            let options = CompressionOptions {
                input_path: input.clone(),
                output_path: output.clone(),
                threads: threads.unwrap_or_else(num_cpus::get),
                level: *level,
                solid: *solid,
            };
            compress_directory(&options)?;
        }
        Commands::Decompress { input, output } => {
            let options = DecompressionOptions {
                input_path: input.clone(),
                output_path: output.clone(),
            };
            decompress_archive(&options)?;
        }
        Commands::CreateImage { input, output, level } => {
            let options = ImageOptions {
                input_path: input.clone(),
                output_path: output.clone(),
                compression_level: *level,
            };
            create_image(&options)?;
        }
        Commands::ExtractImage { input, output } => {
            let options = ExtractOptions {
                image_path: input.clone(),
                output_path: output.clone(),
            };
            extract_image(&options)?;
        }
    }

    Ok(())
}

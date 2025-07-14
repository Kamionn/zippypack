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
mod error;
mod config;
mod metrics;

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::{info, error, warn};
use tracing_subscriber::{fmt, EnvFilter};
use compress::{compress_directory, CompressionOptions};
use decompress::{decompress_archive, DecompressionOptions};
use image::{create_image, extract_image, ImageOptions, ExtractOptions};
use config::Config;
use metrics::Metrics;

#[derive(Parser)]
#[command(name = "zippy")]
#[command(about = "Modern compression tool with deduplication", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbosity level (0-4)
    #[arg(short, long, default_value = "2")]
    verbosity: u8,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Number of threads (overrides config file)
    #[arg(long)]
    threads: Option<usize>,
    
    /// Enable detailed metrics output
    #[arg(long)]
    metrics: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress a directory
    Compress {
        /// Directory to compress
        #[arg(short, long)]
        input: PathBuf,
        /// Output .zpp file
        #[arg(short, long)]
        output: PathBuf,
        /// Compression level (1-22, overrides config)
        #[arg(short = 'l', long)]
        level: Option<i32>,
        /// Solid mode (compress as single stream)
        #[arg(long)]
        solid: bool,
    },
    /// Decompress a .zpp archive
    Decompress {
        /// .zpp archive to decompress
        #[arg(short, long)]
        input: PathBuf,
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Create system image with deduplication
    CreateImage {
        /// Directory to capture
        #[arg(short, long)]
        input: PathBuf,
        /// Output .zpak image file
        #[arg(short, long)]
        output: PathBuf,
        /// Compression level (1-22, overrides config)
        #[arg(short = 'l', long)]
        level: Option<i32>,
    },
    /// Extract system image
    ExtractImage {
        /// .zpak image file to extract
        #[arg(short, long)]
        input: PathBuf,
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize structured logging
    let log_level = match cli.verbosity {
        0 => "error",
        1 => "warn", 
        2 => "info",
        3 => "debug",
        _ => "trace",
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!("zippy={}", log_level)))
        .with_target(false)
        .init();

    info!(version = env!("CARGO_PKG_VERSION"), "ZippyPack starting");

    // Load configuration
    let mut config = if let Some(config_path) = &cli.config {
        info!(config_file = %config_path.display(), "Loading configuration file");
        Config::from_file(config_path).unwrap_or_else(|e| {
            warn!(error = %e, "Failed to load config file, using defaults");
            Config::default()
        })
    } else {
        Config::default()
    };

    // Merge CLI arguments with config
    config.merge_with_cli(None, cli.threads, cli.verbosity >= 3);

    // Initialize metrics if requested
    let metrics = if cli.metrics {
        Some(Metrics::new())
    } else {
        None
    };

    info!(
        compression_level = config.compression_level,
        max_threads = config.max_threads,
        "Configuration loaded"
    );

    match &cli.command {
        Commands::Compress { input, output, level, solid } => {
            let final_level = level.unwrap_or(config.compression_level);
            info!(
                input = %input.display(),
                output = %output.display(),
                level = final_level,
                solid = solid,
                "Starting compression"
            );
            
            let options = CompressionOptions {
                input_path: input.clone(),
                output_path: output.clone(),
                threads: config.max_threads,
                level: final_level,
                solid: *solid,
            };
            
            if let Some(ref m) = metrics { m.start_compression(); }
            let result = compress_directory(&options);
            if let Some(ref m) = metrics { 
                m.end_compression();
                m.print_summary();
            }
            result?;
        }
        Commands::Decompress { input, output } => {
            info!(
                input = %input.display(),
                output = %output.display(),
                "Starting decompression"
            );
            
            let options = DecompressionOptions {
                input_path: input.clone(),
                output_path: output.clone(),
            };
            decompress_archive(&options)?;
        }
        Commands::CreateImage { input, output, level } => {
            let final_level = level.unwrap_or(config.compression_level);
            info!(
                input = %input.display(),
                output = %output.display(),
                level = final_level,
                "Creating system image"
            );
            
            let options = ImageOptions {
                input_path: input.clone(),
                output_path: output.clone(),
                compression_level: final_level,
            };
            
            if let Some(ref m) = metrics { m.start_compression(); }
            let result = create_image(&options);
            if let Some(ref m) = metrics { 
                m.end_compression();
                m.print_summary();
            }
            result?;
        }
        Commands::ExtractImage { input, output } => {
            info!(
                input = %input.display(),
                output = %output.display(),
                "Extracting system image"
            );
            
            let options = ExtractOptions {
                image_path: input.clone(),
                output_path: output.clone(),
            };
            extract_image(&options)?;
        }
    }

    info!("Operation completed successfully");
    Ok(())
}

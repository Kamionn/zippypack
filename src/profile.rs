use std::path::Path;
use log::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)] // Used by compress.rs
pub enum CompressionProfile {
    /// Pour les fichiers déjà compressés (images, vidéos, etc.)
    AlreadyCompressed,
    /// Pour les fichiers texte et code source
    Text,
    /// Pour les fichiers binaires
    Binary,
    /// Pour les fichiers Unity/Unreal Engine
    GameEngine,
}

impl CompressionProfile {
    #[allow(dead_code)] // Used by compress.rs
    pub fn get_compression_level(&self) -> i32 {
        match self {
            Self::AlreadyCompressed => 1, // Pas besoin de compression agressive
            Self::Text => 19, // Compression maximale pour le texte
            Self::Binary => 12, // Bon compromis pour les binaires
            Self::GameEngine => 15, // Compression élevée pour les assets
        }
    }
}

#[allow(dead_code)] // Used by compress.rs
pub fn detect_profile(path: &Path) -> CompressionProfile {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let profile = match extension.as_str() {
        // Fichiers déjà compressés
        "zip" | "rar" | "7z" | "gz" | "bz2" | "xz" | "jpg" | "jpeg" | "png" | "gif" | "mp3" | "mp4" | "avi" => {
            CompressionProfile::AlreadyCompressed
        }
        // Fichiers texte
        "txt" | "md" | "json" | "xml" | "html" | "css" | "js" | "ts" | "py" | "rs" | "c" | "cpp" | "h" | "hpp" => {
            CompressionProfile::Text
        }
        // Fichiers Unity et Unreal
        "unity" | "uasset" | "umap" | "uproject" | "uplugin" | "prefab" | "scene" | "asset" => {
            CompressionProfile::GameEngine
        }
        // Fichiers binaires par défaut
        _ => CompressionProfile::Binary,
    };

    info!("Profil détecté pour {}: {:?}", path.display(), profile);
    profile
}
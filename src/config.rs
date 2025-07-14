use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Default compression level (1-22)
    pub compression_level: i32,
    
    /// Maximum number of threads to use
    pub max_threads: usize,
    
    /// Block size for deduplication (in bytes)
    pub block_size: usize,
    
    /// Memory limit for compression (in MB)
    pub memory_limit: usize,
    
    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compression_level: 22,
            max_threads: num_cpus::get(),
            block_size: 65536, // 64KB
            memory_limit: 1024, // 1GB
            verbose: false,
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to a TOML file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        Ok(())
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if !(1..=22).contains(&self.compression_level) {
            anyhow::bail!("Compression level must be between 1 and 22");
        }
        
        if self.max_threads == 0 || self.max_threads > 1024 {
            anyhow::bail!("Max threads must be between 1 and 1024");
        }
        
        if self.block_size < 1024 || self.block_size > 1024 * 1024 * 1024 {
            anyhow::bail!("Block size must be between 1KB and 1GB");
        }
        
        if self.memory_limit < 64 || self.memory_limit > 64 * 1024 {
            anyhow::bail!("Memory limit must be between 64MB and 64GB");
        }
        
        Ok(())
    }
    
    /// Merge with CLI arguments, giving precedence to CLI
    pub fn merge_with_cli(&mut self, cli_level: Option<i32>, cli_threads: Option<usize>, cli_verbose: bool) {
        if let Some(level) = cli_level {
            self.compression_level = level;
        }
        
        if let Some(threads) = cli_threads {
            self.max_threads = threads;
        }
        
        if cli_verbose {
            self.verbose = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.compression_level, 22);
        assert!(config.max_threads > 0);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Test invalid compression level
        config.compression_level = 0;
        assert!(config.validate().is_err());
        
        config.compression_level = 23;
        assert!(config.validate().is_err());
        
        // Test invalid threads
        config.compression_level = 10;
        config.max_threads = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.compression_level, parsed.compression_level);
        assert_eq!(config.max_threads, parsed.max_threads);
    }
    
    #[test]
    fn test_config_file_operations() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let config = Config::default();
        config.save_to_file(&config_path).unwrap();
        
        let loaded_config = Config::from_file(&config_path).unwrap();
        assert_eq!(config.compression_level, loaded_config.compression_level);
    }
}
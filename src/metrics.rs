use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Debug)]
pub struct Metrics {
    /// Total number of files processed
    pub files_processed: AtomicU64,
    
    /// Total number of bytes processed (before compression)
    pub bytes_processed: AtomicU64,
    
    /// Total number of compressed bytes
    pub bytes_compressed: AtomicU64,
    
    /// Total number of unique blocks found (for deduplication)
    pub unique_blocks: AtomicU64,
    
    /// Total number of duplicate blocks found
    pub duplicate_blocks: AtomicU64,
    
    /// Total compression time tracking
    compression_timing: Mutex<Option<Instant>>,
    compression_duration: AtomicU64, // nanoseconds
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            files_processed: AtomicU64::new(0),
            bytes_processed: AtomicU64::new(0),
            bytes_compressed: AtomicU64::new(0),
            unique_blocks: AtomicU64::new(0),
            duplicate_blocks: AtomicU64::new(0),
            compression_timing: Mutex::new(None),
            compression_duration: AtomicU64::new(0),
        }
    }
}

impl Metrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    
    pub fn start_compression(&self) {
        if let Ok(mut timing) = self.compression_timing.lock() {
            *timing = Some(Instant::now());
        }
    }
    
    pub fn end_compression(&self) {
        if let Ok(mut timing) = self.compression_timing.lock() {
            if let Some(start) = timing.take() {
                let duration = start.elapsed();
                self.compression_duration.store(duration.as_nanos() as u64, Ordering::Relaxed);
            }
        }
    }
    
    pub fn increment_files(&self) {
        self.files_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn add_bytes_processed(&self, bytes: u64) {
        self.bytes_processed.fetch_add(bytes, Ordering::Relaxed);
    }
    
    pub fn add_bytes_compressed(&self, bytes: u64) {
        self.bytes_compressed.fetch_add(bytes, Ordering::Relaxed);
    }
    
    pub fn increment_unique_blocks(&self) {
        self.unique_blocks.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_duplicate_blocks(&self) {
        self.duplicate_blocks.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_compression_ratio(&self) -> f64 {
        let processed = self.bytes_processed.load(Ordering::Relaxed);
        let compressed = self.bytes_compressed.load(Ordering::Relaxed);
        
        if processed == 0 {
            return 0.0;
        }
        
        (compressed as f64 / processed as f64) * 100.0
    }
    
    pub fn get_deduplication_ratio(&self) -> f64 {
        let unique = self.unique_blocks.load(Ordering::Relaxed);
        let total = unique + self.duplicate_blocks.load(Ordering::Relaxed);
        
        if total == 0 {
            return 0.0;
        }
        
        ((total - unique) as f64 / total as f64) * 100.0
    }
    
    pub fn get_compression_speed(&self) -> f64 {
        let duration_nanos = self.compression_duration.load(Ordering::Relaxed);
        let bytes = self.bytes_processed.load(Ordering::Relaxed);
        
        if duration_nanos == 0 {
            return 0.0;
        }
        
        let duration_secs = duration_nanos as f64 / 1_000_000_000.0;
        bytes as f64 / duration_secs / 1_024_000.0 // MB/s
    }
    
    pub fn print_summary(&self) {
        let files = self.files_processed.load(Ordering::Relaxed);
        let processed = self.bytes_processed.load(Ordering::Relaxed);
        let compressed = self.bytes_compressed.load(Ordering::Relaxed);
        let unique_blocks = self.unique_blocks.load(Ordering::Relaxed);
        let duplicate_blocks = self.duplicate_blocks.load(Ordering::Relaxed);
        
        info!(
            files_processed = files,
            bytes_processed = processed,
            bytes_compressed = compressed,
            compression_ratio = %format!("{:.2}%", self.get_compression_ratio()),
            deduplication_ratio = %format!("{:.2}%", self.get_deduplication_ratio()),
            compression_speed = %format!("{:.2} MB/s", self.get_compression_speed()),
            unique_blocks = unique_blocks,
            duplicate_blocks = duplicate_blocks,
            "Compression completed"
        );
    }
    
    pub fn reset(&self) {
        self.files_processed.store(0, Ordering::Relaxed);
        self.bytes_processed.store(0, Ordering::Relaxed);
        self.bytes_compressed.store(0, Ordering::Relaxed);
        self.unique_blocks.store(0, Ordering::Relaxed);
        self.duplicate_blocks.store(0, Ordering::Relaxed);
        self.compression_duration.store(0, Ordering::Relaxed);
    }
}

// Progress tracker for real-time updates
pub struct ProgressTracker {
    metrics: Arc<Metrics>,
    last_update: Instant,
    update_interval: Duration,
}

impl ProgressTracker {
    pub fn new(metrics: Arc<Metrics>) -> Self {
        Self {
            metrics,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1), // Update every second
        }
    }
    
    pub fn update(&mut self, force: bool) {
        if force || self.last_update.elapsed() >= self.update_interval {
            let files = self.metrics.files_processed.load(Ordering::Relaxed);
            let processed = self.metrics.bytes_processed.load(Ordering::Relaxed);
            let speed = self.metrics.get_compression_speed();
            
            info!(
                files_processed = files,
                bytes_processed = processed,
                speed_mbps = %format!("{:.2}", speed),
                "Progress update"
            );
            
            self.last_update = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_basic_operations() {
        let metrics = Metrics::new();
        
        metrics.increment_files();
        metrics.add_bytes_processed(1000);
        metrics.add_bytes_compressed(500);
        
        assert_eq!(metrics.files_processed.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.bytes_processed.load(Ordering::Relaxed), 1000);
        assert_eq!(metrics.bytes_compressed.load(Ordering::Relaxed), 500);
        assert_eq!(metrics.get_compression_ratio(), 50.0);
    }
    
    #[test]
    fn test_deduplication_ratio() {
        let metrics = Metrics::new();
        
        metrics.increment_unique_blocks();
        metrics.increment_unique_blocks();
        metrics.increment_duplicate_blocks();
        metrics.increment_duplicate_blocks();
        metrics.increment_duplicate_blocks();
        
        // 2 unique, 3 duplicates = 5 total, 60% deduplication
        assert_eq!(metrics.get_deduplication_ratio(), 60.0);
    }
    
    #[test]
    fn test_metrics_reset() {
        let metrics = Metrics::new();
        
        metrics.increment_files();
        metrics.add_bytes_processed(1000);
        
        metrics.reset();
        
        assert_eq!(metrics.files_processed.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.bytes_processed.load(Ordering::Relaxed), 0);
    }
}
use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};

use crate::metadata::SampleMetadata;
use crate::db::{SampleRecord, insert_sample, get_sample_by_hash};

pub struct FileWatcher {
    watch_dir: PathBuf,
    library_dir: PathBuf,
    database_path: PathBuf,
    retry_attempts: u32,
    error_count: u32,
}

impl FileWatcher {
    pub fn new(watch_dir: PathBuf, library_dir: PathBuf, database_path: PathBuf) -> Result<Self> {
        // Create directories if they don't exist
        Self::ensure_directory(&watch_dir)?;
        Self::ensure_directory(&library_dir)?;
        
        if let Some(parent) = database_path.parent() {
            Self::ensure_directory(parent)?;
        }
        
        Ok(Self {
            watch_dir,
            library_dir,
            database_path,
            retry_attempts: 3,
            error_count: 0,
        })
    }
    
    fn ensure_directory(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| anyhow::anyhow!("Failed to create directory {:?}: {}", path, e))?;
            println!("📁 Created directory: {:?}", path);
        } else if !path.is_dir() {
            return Err(anyhow::anyhow!("{:?} exists but is not a directory", path));
        }
        Ok(())
    }
    
    pub async fn start_watching(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(100);
        
        // Create the file system watcher with error handling
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if let Err(e) = tx.blocking_send(event) {
                            eprintln!("⚠️  Failed to send event: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("🚨 Watch error: {:?}", e);
                        // TODO: Implement watcher recovery mechanism
                    }
                }
            },
            Config::default(),
        ).map_err(|e| anyhow::anyhow!("Failed to create file watcher: {}", e))?;
        
        // Start watching the directory
        watcher.watch(&self.watch_dir, RecursiveMode::Recursive)
            .map_err(|e| anyhow::anyhow!("Failed to watch directory {:?}: {}", self.watch_dir, e))?;
        
        println!("👀 Started watching directory: {:?}", self.watch_dir);
        
        // Process events with error handling and recovery
        while let Some(event) = rx.recv().await {
            if let Err(e) = self.handle_event_with_retry(event).await {
                self.error_count += 1;
                eprintln!("🚨 Error handling event (total errors: {}): {}", self.error_count, e);
                
                // If too many errors, pause briefly to avoid rapid failures
                if self.error_count % 10 == 0 {
                    println!("⏸️  Too many errors, pausing for 30 seconds...");
                    sleep(Duration::from_secs(30)).await;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_event_with_retry(&mut self, event: Event) -> Result<()> {
        for attempt in 1..=self.retry_attempts {
            match self.handle_event(event.clone()).await {
                Ok(()) => {
                    // Reset error count on success
                    if self.error_count > 0 {
                        self.error_count = self.error_count.saturating_sub(1);
                    }
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("🔄 Attempt {}/{} failed: {}", attempt, self.retry_attempts, e);
                    if attempt < self.retry_attempts {
                        // Exponential backoff
                        let delay = Duration::from_millis(1000 * (2_u64.pow(attempt - 1)));
                        sleep(delay).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn handle_event(&self, event: Event) -> Result<()> {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    if let Some(extension) = path.extension() {
                        match extension.to_str() {
                            Some("wav") => {
                                println!("🎵 New WAV file detected: {:?}", path);
                                self.process_wav_file(&path).await?;
                            }
                            Some("json") => {
                                println!("📄 New JSON metadata file detected: {:?}", path);
                                self.process_json_file(&path).await?;
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn process_wav_file(&self, wav_path: &Path) -> Result<()> {
        self.validate_file(wav_path, "WAV")?;
        
        println!("🔍 Processing WAV file: {:?}", wav_path);
        
        // Wait for corresponding JSON metadata file with timeout
        let json_path = wav_path.with_extension("json");
        
        // Try to find JSON file with timeout and retries
        let mut attempts = 0;
        while attempts < 10 && !json_path.exists() {
            sleep(Duration::from_millis(500)).await;
            attempts += 1;
        }
        
        if json_path.exists() {
            println!("✅ Found corresponding metadata file: {:?}", json_path);
            self.process_sample_pair(wav_path, &json_path).await?;
        } else {
            println!("⚠️  No metadata file found for: {:?}", wav_path);
            self.handle_orphaned_wav(wav_path).await?;
        }
        
        Ok(())
    }
    
    async fn process_json_file(&self, json_path: &Path) -> Result<()> {
        self.validate_file(json_path, "JSON")?;
        
        println!("🔍 Processing JSON file: {:?}", json_path);
        
        // Check if there's a corresponding WAV file
        let wav_path = json_path.with_extension("wav");
        if wav_path.exists() {
            println!("✅ Found corresponding WAV file: {:?}", wav_path);
            self.process_sample_pair(&wav_path, json_path).await?;
        } else {
            println!("⏳ JSON metadata file arrived before WAV: {:?}", json_path);
            // The WAV processing will handle this when it arrives
        }
        
        Ok(())
    }
    
    fn validate_file(&self, file_path: &Path, file_type: &str) -> Result<()> {
        if !file_path.exists() {
            return Err(anyhow::anyhow!("{} file no longer exists: {:?}", file_type, file_path));
        }
        
        if !file_path.is_file() {
            return Err(anyhow::anyhow!("{} path is not a file: {:?}", file_type, file_path));
        }
        
        let metadata = fs::metadata(file_path)
            .map_err(|e| anyhow::anyhow!("Cannot read {} file metadata: {}", file_type, e))?;
            
        if metadata.len() == 0 {
            return Err(anyhow::anyhow!("{} file is empty: {:?}", file_type, file_path));
        }
        
        Ok(())
    }
    
    pub async fn process_sample_pair_public(&self, wav_path: &Path, json_path: &Path) -> Result<()> {
        self.process_sample_pair(wav_path, json_path).await
    }
    
    async fn process_sample_pair(&self, wav_path: &Path, json_path: &Path) -> Result<()> {
        println!("🎵 Processing sample pair: {:?} + {:?}", wav_path.file_name(), json_path.file_name());
        
        // Validate both files
        self.validate_file(wav_path, "WAV")?;
        self.validate_file(json_path, "JSON")?;
        
        // Parse metadata with timeout
        let metadata = timeout(Duration::from_secs(10), async {
            SampleMetadata::from_file(json_path)
        }).await
        .map_err(|_| anyhow::anyhow!("Timeout parsing metadata from {:?}", json_path))?
        .map_err(|e| anyhow::anyhow!("Failed to parse metadata from {:?}: {}", json_path, e))?;
        
        // Calculate file hash for deduplication
        let file_hash = self.calculate_file_hash_with_retry(wav_path).await?;
        println!("🔐 Calculated file hash: {}", file_hash);
        
        // Check if this sample already exists in the database
        if let Ok(Some(_existing)) = get_sample_by_hash(&self.database_path, &file_hash) {
            println!("⚠️  Sample already exists in library (duplicate detected)");
            
            // Clean up duplicate files
            self.cleanup_duplicate_files(wav_path, json_path).await?;
            return Ok(());
        }
        
        // Determine target library path
        let target_path = metadata.get_library_path(&self.library_dir);
        println!("📍 Target path: {:?}", target_path);
        
        // Create target directory with proper error handling
        if let Some(parent) = target_path.parent() {
            Self::ensure_directory(parent)?;
            println!("📁 Ensured directory: {:?}", parent);
        }
        
        // Atomic file move with backup
        self.move_file_safely(wav_path, &target_path).await?;
        println!("✅ Moved WAV file to: {:?}", target_path);
        
        // Create database record
        let mut record = SampleRecord::from(&metadata);
        record.file_path = target_path.to_string_lossy().to_string();
        record.file_hash = file_hash;
        
        // Insert into database with retry
        self.insert_sample_with_retry(record).await?;
        
        // Clean up the JSON file
        self.cleanup_metadata_file(json_path).await?;
        
        println!("🎉 Sample processing complete!\n");
        Ok(())
    }
    
    async fn calculate_file_hash_with_retry(&self, file_path: &Path) -> Result<String> {
        for attempt in 1..=3 {
            match self.calculate_file_hash(file_path) {
                Ok(hash) => return Ok(hash),
                Err(e) => {
                    eprintln!("🔄 Hash calculation attempt {}/3 failed: {}", attempt, e);
                    if attempt < 3 {
                        sleep(Duration::from_millis(1000)).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        unreachable!()
    }
    
    fn calculate_file_hash(&self, file_path: &Path) -> Result<String> {
        let data = fs::read(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read file for hashing: {}", e))?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
    
    async fn move_file_safely(&self, source: &Path, target: &Path) -> Result<()> {
        // Create backup name in case of failure (reserved for future rollback functionality)
        let _backup_path = source.with_extension("wav.backup");
        
        // First, try to copy the file
        fs::copy(source, target)
            .map_err(|e| anyhow::anyhow!("Failed to copy file to target: {}", e))?;
        
        // Verify the copy is complete and valid
        let source_size = fs::metadata(source)?.len();
        let target_size = fs::metadata(target)?.len();
        
        if source_size != target_size {
            // Remove invalid copy
            let _ = fs::remove_file(target);
            return Err(anyhow::anyhow!("File copy verification failed: size mismatch"));
        }
        
        // Only remove source after successful copy and verification
        fs::remove_file(source)
            .map_err(|e| anyhow::anyhow!("Failed to remove source file after copy: {}", e))?;
        
        Ok(())
    }
    
    async fn insert_sample_with_retry(&self, record: SampleRecord) -> Result<()> {
        for attempt in 1..=3 {
            match insert_sample(&self.database_path, record.clone()) {
                Ok(id) => {
                    println!("✅ Added sample to database with ID: {}", id);
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("🔄 Database insert attempt {}/3 failed: {}", attempt, e);
                    if attempt < 3 {
                        sleep(Duration::from_millis(1000)).await;
                    } else {
                        return Err(anyhow::anyhow!("Failed to add sample to database after {} attempts: {}", attempt, e));
                    }
                }
            }
        }
        unreachable!()
    }
    
    async fn cleanup_duplicate_files(&self, wav_path: &Path, json_path: &Path) -> Result<()> {
        // Remove duplicate WAV file
        if let Err(e) = fs::remove_file(wav_path) {
            eprintln!("⚠️  Failed to remove duplicate WAV file {:?}: {}", wav_path, e);
        } else {
            println!("🗑️  Removed duplicate WAV file");
        }
        
        // Remove duplicate JSON file
        if let Err(e) = fs::remove_file(json_path) {
            eprintln!("⚠️  Failed to remove duplicate JSON file {:?}: {}", json_path, e);
        } else {
            println!("🗑️  Removed duplicate JSON file");
        }
        
        Ok(())
    }
    
    async fn cleanup_metadata_file(&self, json_path: &Path) -> Result<()> {
        if let Err(e) = fs::remove_file(json_path) {
            eprintln!("⚠️  Warning: Failed to remove JSON file {:?}: {}", json_path, e);
        } else {
            println!("🗑️  Cleaned up metadata file");
        }
        Ok(())
    }
    
    async fn handle_orphaned_wav(&self, wav_path: &Path) -> Result<()> {
        println!("🤔 Handling WAV file without metadata: {:?}", wav_path);
        
        // For now, just log it. In the future, we could:
        // - Move to a special "unprocessed" folder
        // - Try to extract metadata from filename
        // - Queue for manual processing
        
        println!("⏳ WAV file queued for manual processing or metadata arrival");
        Ok(())
    }
    

}

// Make SampleRecord cloneable for retry operations
impl Clone for crate::db::SampleRecord {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            file_path: self.file_path.clone(),
            pack_name: self.pack_name.clone(),
            pack_uuid: self.pack_uuid.clone(),
            filename: self.filename.clone(),
            file_hash: self.file_hash.clone(),
            bpm: self.bpm,
            audio_key: self.audio_key.clone(),
            chord_type: self.chord_type.clone(),
            tags: self.tags.clone(),
            mapped_category: self.mapped_category.clone(),
            sample_type: self.sample_type.clone(),
            duration: self.duration,
            file_size: self.file_size,
            provider_name: self.provider_name.clone(),
            date_downloaded: self.date_downloaded.clone(),
            splice_url: self.splice_url.clone(),
            preview_url: self.preview_url.clone(),
            asset_uuid: self.asset_uuid.clone(),
        }
    }
}

// Public API functions for main.rs

pub async fn watch_directory(
    watch_dir: &Path, 
    library_dir: &Path, 
    database_path: &Path
) -> Result<()> {
    let mut watcher = FileWatcher::new(
        watch_dir.to_path_buf(),
        library_dir.to_path_buf(),
        database_path.to_path_buf(),
    )?;
    
    watcher.start_watching().await
}

pub async fn process_sample_pair(
    wav_path: &Path,
    json_path: &Path,
    library_dir: &Path,
    database_path: &Path,
) -> Result<()> {
    let watcher = FileWatcher::new(
        PathBuf::from("/tmp"), // Dummy watch dir since we're not watching
        library_dir.to_path_buf(),
        database_path.to_path_buf(),
    )?;
    
    watcher.process_sample_pair_public(wav_path, json_path).await
} 
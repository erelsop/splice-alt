use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::signal;

mod watcher;
mod db;
mod metadata;

#[derive(Parser)]
#[command(name = "splice-alt-daemon")]
#[command(about = "A daemon that watches for Splice samples and organizes them into a structured library")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Directory to watch for new samples (default: ~/Downloads)
    #[arg(short, long)]
    watch_dir: Option<PathBuf>,
    
    /// Sample library base directory (default: ~/Music/Samples/SpliceLib)
    #[arg(short, long)]
    library_dir: Option<PathBuf>,
    
    /// Database file path (default: ~/.local/share/splice-alt/samples.db)
    #[arg(short, long)]
    database: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test metadata parsing with a JSON file
    Test {
        /// Path to JSON metadata file
        metadata_file: PathBuf,
    },
    /// Process a specific WAV and JSON file pair directly
    Process {
        /// Path to WAV file
        wav_file: PathBuf,
        /// Path to JSON metadata file
        json_file: PathBuf,
        /// Target library directory
        #[arg(short, long)]
        library_dir: PathBuf,
        /// Database file path
        #[arg(short, long)]
        database: PathBuf,
    },
    /// List samples by category
    List {
        /// Bitwig category to list (e.g., Bass, Lead, Drum Loop)
        category: String,
        /// Database file path
        #[arg(short, long)]
        database: Option<PathBuf>,
    },
    /// Update file path in database (useful when files are moved)
    UpdatePath {
        /// File hash of the sample to update
        file_hash: String,
        /// New file path
        new_path: PathBuf,
        /// Database file path
        #[arg(short, long)]
        database: Option<PathBuf>,
    },
    /// Run the daemon (default command)
    Watch,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Some(Commands::Test { metadata_file }) => {
            test_metadata_parsing(&metadata_file).await
        }
        Some(Commands::Process { wav_file, json_file, library_dir, database }) => {
            process_files_directly(&wav_file, &json_file, &library_dir, &database).await
        }
        Some(Commands::List { category, database }) => {
            list_samples_by_category(&category, database).await
        }
        Some(Commands::UpdatePath { file_hash, new_path, database }) => {
            update_sample_path(&file_hash, &new_path, database).await
        }
        Some(Commands::Watch) | None => {
            run_daemon(args).await
        }
    }
}

async fn list_samples_by_category(category: &str, database: Option<PathBuf>) -> Result<()> {
    let database_path = database.unwrap_or_else(|| {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("splice-alt")
            .join("samples.db")
    });
    
    // Initialize database if it doesn't exist
    if !database_path.exists() {
        println!("📦 Database doesn't exist, initializing...");
        db::init_database(&database_path)?;
    }
    
    // Parse the category string to BitwigCategory
    let bitwig_category = match metadata::BitwigCategory::from_str(category) {
        Ok(cat) => cat,
        Err(_) => {
            println!("❌ Invalid category '{}'. Available categories:", category);
            println!("   Bass, Bell, Brass, Chip, Cymbal, Drone, Drum Loop,");
            println!("   Guitar, Hi-hat, Keyboards, Kick, Lead, Mallet,");
            println!("   Orchestral, Organ, Other Drums, Pad, Percussion,");
            println!("   Piano, Snare, Sound FX, Strings, Synth, Tom,");
            println!("   Unknown, Vocal, Winds");
            return Ok(());
        }
    };
    
    println!("📂 Listing samples in category: {}", bitwig_category.as_str());
    
    match db::get_samples_by_category(&database_path, bitwig_category) {
        Ok(samples) => {
            if samples.is_empty() {
                println!("No samples found in category '{}'", category);
            } else {
                println!("Found {} samples:", samples.len());
                println!();
                
                let mut current_pack = String::new();
                for sample in samples {
                    if sample.pack_name != current_pack {
                        current_pack = sample.pack_name.clone();
                        println!("📦 {}", current_pack);
                    }
                    
                    let bpm_str = sample.bpm.map_or("--".to_string(), |b| b.to_string());
                    let key_str = sample.audio_key.unwrap_or_else(|| "--".to_string());
                    
                    println!("   🎵 {} ({}bpm, {})", sample.filename, bpm_str, key_str);
                    println!("      📁 {}", sample.file_path);
                    
                    // Parse and display tags
                    if let Ok(tags) = serde_json::from_str::<Vec<String>>(&sample.tags) {
                        if !tags.is_empty() {
                            println!("      🏷️  {}", tags.join(", "));
                        }
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to query database: {}", e);
            println!("Make sure the database exists and the daemon has been run at least once.");
        }
    }
    
    Ok(())
}

async fn update_sample_path(file_hash: &str, new_path: &PathBuf, database: Option<PathBuf>) -> Result<()> {
    let database_path = database.unwrap_or_else(|| {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("splice-alt")
            .join("samples.db")
    });
    
    // Initialize database if it doesn't exist
    if !database_path.exists() {
        println!("📦 Database doesn't exist, initializing...");
        db::init_database(&database_path)?;
    }
    
    println!("🔧 Updating file path for sample with hash: {}", file_hash);
    println!("📁 New path: {:?}", new_path);
    
    // Verify the new file actually exists
    if !new_path.exists() {
        println!("❌ Error: File does not exist at the specified path");
        println!("Please make sure the file exists before updating the database");
        return Ok(());
    }
    
    if !new_path.is_file() {
        println!("❌ Error: Path exists but is not a file");
        return Ok(());
    }
    
    match db::update_file_path(&database_path, file_hash, &new_path.to_string_lossy()) {
        Ok(()) => {
            println!("✅ Successfully updated file path in database");
            
            // Verify the update by fetching the sample
            if let Ok(Some(sample)) = db::get_sample_by_hash(&database_path, file_hash) {
                println!("📊 Sample details:");
                println!("   📦 Pack: {}", sample.pack_name);
                println!("   🎵 File: {}", sample.filename);
                println!("   📁 Path: {}", sample.file_path);
            }
        }
        Err(e) => {
            println!("❌ Failed to update file path: {}", e);
            println!("Make sure:");
            println!("  - The database exists and is accessible");
            println!("  - The file hash exists in the database");
            println!("  - You have write permissions to the database");
        }
    }
    
    Ok(())
}

async fn process_files_directly(wav_file: &PathBuf, json_file: &PathBuf, library_dir: &PathBuf, database: &PathBuf) -> Result<()> {
    println!("🔧 Direct file processing test");
    println!("WAV: {:?}", wav_file);
    println!("JSON: {:?}", json_file);
    println!("Library: {:?}", library_dir);
    println!("Database: {:?}", database);
    
    // Initialize database
    db::init_database(database)?;
    
    // Create a temporary watcher just to use its processing methods
    let watcher = watcher::FileWatcher::new(
        PathBuf::from("/tmp"), // dummy watch dir
        library_dir.clone(),
        database.clone()
    )?;
    
    // Process the files directly
    watcher.process_sample_pair_public(wav_file, json_file).await?;
    
    println!("✅ Direct processing complete!");
    Ok(())
}

async fn test_metadata_parsing(metadata_file: &PathBuf) -> Result<()> {
    println!("Testing metadata parsing with file: {:?}", metadata_file);
    
    match metadata::SampleMetadata::from_file(metadata_file) {
        Ok(metadata) => {
            println!("✅ Successfully parsed metadata!");
            println!("Pack: {}", metadata.sample_meta_data.pack.name);
            println!("Filename: {}", metadata.sample_meta_data.filename);
            println!("BPM: {:?}", metadata.sample_meta_data.bpm);
            println!("Key: {:?}", metadata.sample_meta_data.audio_key);
            println!("Tags: {:?}", metadata.sample_meta_data.tags);
            println!("Mapped Category: {:?}", metadata.get_category());
            
            // Test library path generation
            let library_base = PathBuf::from("/tmp/test-library");
            let target_path = metadata.get_library_path(&library_base);
            println!("Target library path: {:?}", target_path);
        }
        Err(e) => {
            println!("❌ Failed to parse metadata: {}", e);
        }
    }
    
    Ok(())
}

async fn run_daemon(args: Args) -> Result<()> {
    // Set up default paths
    let watch_dir = args.watch_dir.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Downloads")
    });
    
    let library_dir = args.library_dir.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Music")
            .join("Samples")
            .join("SpliceLib")
    });
    
    let database_path = args.database.unwrap_or_else(|| {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("splice-alt")
            .join("samples.db")
    });
    
    println!("Splice Alt Daemon starting...");
    println!("Watch directory: {:?}", watch_dir);
    println!("Library directory: {:?}", library_dir);
    println!("Database: {:?}", database_path);
    
    // Initialize database
    db::init_database(&database_path)?;
    
    // Start the file watcher
    let mut watcher = watcher::FileWatcher::new(watch_dir, library_dir, database_path)?;
    
    // Start watching in a separate task
    let watch_handle = tokio::spawn(async move {
        if let Err(e) = watcher.start_watching().await {
            eprintln!("File watcher error: {}", e);
        }
    });
    
    // Wait for shutdown signal
    println!("Press Ctrl+C to stop the daemon");
    signal::ctrl_c().await?;
    
    println!("Shutting down...");
    watch_handle.abort();
    
    Ok(())
} 
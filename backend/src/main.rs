use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::signal;
use std::process::{Command, Stdio};
use std::fs;
use std::env;

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
    /// Start the daemon in the background
    Start,
    /// Stop the background daemon
    Stop,
    /// Check daemon status
    Status,
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

fn get_pid_file_path() -> PathBuf {
    dirs::runtime_dir()
        .or_else(|| dirs::cache_dir())
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("splice-alt-daemon.pid")
}

fn get_log_file_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("splice-alt-daemon.log")
}

fn get_current_executable() -> Result<PathBuf> {
    Ok(env::current_exe()?)
}

fn read_pid_file() -> Option<u32> {
    let pid_file = get_pid_file_path();
    if pid_file.exists() {
        if let Ok(content) = fs::read_to_string(&pid_file) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                return Some(pid);
            }
        }
    }
    None
}

fn write_pid_file(pid: u32) -> Result<()> {
    let pid_file = get_pid_file_path();
    if let Some(parent) = pid_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&pid_file, pid.to_string())?;
    Ok(())
}

fn remove_pid_file() -> Result<()> {
    let pid_file = get_pid_file_path();
    if pid_file.exists() {
        fs::remove_file(&pid_file)?;
    }
    Ok(())
}

fn is_process_running(pid: u32) -> bool {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;
    
    match signal::kill(Pid::from_raw(pid as i32), None) {
        Ok(_) => true,  // Process exists
        Err(_) => false, // Process doesn't exist
    }
}

fn start_daemon(args: Args) -> Result<()> {
    // Check if daemon is already running
    if let Some(pid) = read_pid_file() {
        if is_process_running(pid) {
            println!("✅ Daemon is already running (PID: {})", pid);
            println!("   Use --stop to stop it first, or --status to check");
            return Ok(());
        } else {
            println!("🧹 Cleaning up stale PID file...");
            let _ = remove_pid_file();
        }
    }
    
    let executable = get_current_executable()?;
    let log_file = get_log_file_path();
    
    println!("🚀 Starting Splice Alt daemon in background...");
    println!("📁 Log file: {:?}", log_file);
    
    // Prepare command arguments
    let mut cmd = Command::new(&executable);
    cmd.arg("watch");
    
    if let Some(watch_dir) = args.watch_dir {
        cmd.arg("--watch-dir").arg(watch_dir);
    }
    
    if let Some(library_dir) = args.library_dir {
        cmd.arg("--library-dir").arg(library_dir);
    }
    
    if let Some(database) = args.database {
        cmd.arg("--database").arg(database);
    }
    
    // Set up background process
    let log_file_for_stdout = fs::File::create(&log_file)?;
    let log_file_for_stderr = log_file_for_stdout.try_clone()?;
    
    cmd.stdout(Stdio::from(log_file_for_stdout))
       .stderr(Stdio::from(log_file_for_stderr))
       .stdin(Stdio::null());
    
    // Start the process
    let child = cmd.spawn()?;
    let pid = child.id();
    
    // Write PID file
    write_pid_file(pid)?;
    
    // Give it a moment to start
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Verify it's still running
    if is_process_running(pid) {
        println!("✅ Daemon started successfully (PID: {})", pid);
        println!("   Monitor logs: tail -f {:?}", log_file);
        println!("   Stop daemon: {} --stop", executable.file_name().unwrap().to_string_lossy());
    } else {
        println!("❌ Daemon failed to start. Check logs: {:?}", log_file);
        let _ = remove_pid_file();
    }
    
    Ok(())
}

fn stop_daemon() -> Result<()> {
    if let Some(pid) = read_pid_file() {
        if is_process_running(pid) {
            println!("🛑 Stopping daemon (PID: {})...", pid);
            
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;
            
            // Send SIGTERM first
            if let Err(e) = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                println!("❌ Failed to send SIGTERM: {}", e);
                return Ok(());
            }
            
            // Wait a bit for graceful shutdown
            for i in 0..10 {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if !is_process_running(pid) {
                    println!("✅ Daemon stopped successfully");
                    remove_pid_file()?;
                    return Ok(());
                }
                if i == 4 {
                    println!("⏳ Waiting for graceful shutdown...");
                }
            }
            
            // Force kill if still running
            println!("⚠️  Forcing daemon shutdown...");
            if let Err(e) = signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
                println!("❌ Failed to force kill: {}", e);
            } else {
                println!("✅ Daemon force stopped");
            }
            
            remove_pid_file()?;
        } else {
            println!("🧹 Daemon not running, cleaning up PID file");
            remove_pid_file()?;
        }
    } else {
        println!("ℹ️  Daemon is not running");
    }
    
    Ok(())
}

fn check_daemon_status() -> Result<()> {
    let pid_file = get_pid_file_path();
    let log_file = get_log_file_path();
    
    println!("🔍 Splice Alt Daemon Status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if let Some(pid) = read_pid_file() {
        if is_process_running(pid) {
            println!("✅ Status: Running");
            println!("🆔 PID: {}", pid);
            
            // Try to get process info
            if let Ok(output) = Command::new("ps").args(&["-p", &pid.to_string(), "-o", "pid,ppid,etime,cmd"]).output() {
                if output.status.success() {
                    let ps_output = String::from_utf8_lossy(&output.stdout);
                    let lines: Vec<&str> = ps_output.lines().collect();
                    if lines.len() > 1 {
                        println!("⏱️  Runtime: {}", lines[1].split_whitespace().nth(2).unwrap_or("unknown"));
                    }
                }
            }
        } else {
            println!("❌ Status: Not running (stale PID file)");
            println!("🧹 Cleaning up stale PID file...");
            let _ = remove_pid_file();
        }
    } else {
        println!("⭕ Status: Not running");
    }
    
    println!("📁 PID file: {:?}", pid_file);
    println!("📄 Log file: {:?}", log_file);
    
    if log_file.exists() {
        if let Ok(metadata) = fs::metadata(&log_file) {
            println!("📊 Log size: {} bytes", metadata.len());
            
            // Show last few lines of log
            if let Ok(output) = Command::new("tail").args(&["-n", "5", &log_file.to_string_lossy()]).output() {
                if output.status.success() && !output.stdout.is_empty() {
                    println!("📋 Recent log entries:");
                    for line in String::from_utf8_lossy(&output.stdout).lines() {
                        println!("   {}", line);
                    }
                }
            }
        }
    } else {
        println!("📄 Log file: Not found");
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Some(Commands::Start) => {
            start_daemon(args)
        }
        Some(Commands::Stop) => {
            stop_daemon()
        }
        Some(Commands::Status) => {
            check_daemon_status()
        }
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
    
    // Process the files
    watcher::process_sample_pair(wav_file, json_file, library_dir, database).await
}

async fn test_metadata_parsing(metadata_file: &PathBuf) -> Result<()> {
    println!("🧪 Testing metadata parsing");
    println!("File: {:?}", metadata_file);
    
    if !metadata_file.exists() {
        println!("❌ File does not exist");
        return Ok(());
    }
    
    let content = std::fs::read_to_string(metadata_file)?;
    println!("📄 Raw JSON content:");
    println!("{}", content);
    println!();
    
    match serde_json::from_str::<metadata::SampleMetadata>(&content) {
        Ok(metadata) => {
            println!("✅ Successfully parsed metadata:");
            println!("📦 Pack: {}", metadata.sample.pack.name);
            println!("🎵 File: {}", metadata.sample.name);
            println!("🎯 BPM: {:?}", metadata.sample.bpm);
            println!("🎼 Key: {:?}", metadata.sample.key);
            println!("🏷️  Tags: {:?}", metadata.sample.tags);
            
            // Test category mapping
            let category = metadata::map_tags_to_category(&metadata.sample.tags);
            println!("📂 Mapped category: {}", category.as_str());
        }
        Err(e) => {
            println!("❌ Failed to parse metadata: {}", e);
            println!("Make sure the JSON file contains valid Splice metadata");
        }
    }
    
    Ok(())
}

async fn run_daemon(args: Args) -> Result<()> {
    let watch_dir = args.watch_dir.unwrap_or_else(|| {
        dirs::download_dir().unwrap_or_else(|| PathBuf::from("./downloads"))
    });
    
    let library_dir = args.library_dir.unwrap_or_else(|| {
        dirs::audio_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
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
    
    println!("🎵 Splice Alt Daemon Starting");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("👀 Watching: {:?}", watch_dir);
    println!("📚 Library: {:?}", library_dir);
    println!("🗄️  Database: {:?}", database_path);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Initialize database
    db::init_database(&database_path)?;
    
    // Start the watcher
    let watcher_handle = tokio::spawn(async move {
        if let Err(e) = watcher::watch_directory(&watch_dir, &library_dir, &database_path).await {
            eprintln!("❌ Watcher error: {}", e);
        }
    });
    
    // Wait for shutdown signal
    println!("✅ Daemon is running. Press Ctrl+C to stop.");
    
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("\n🛑 Received shutdown signal");
        }
        _ = watcher_handle => {
            println!("👀 Watcher task completed");
        }
    }
    
    println!("👋 Daemon stopping...");
    Ok(())
}

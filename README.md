# Splice Alt - Automatic Sample Library Organizer

A complete system for automatically organizing Splice samples into a structured library on download.

## 🎯 Overview

Splice Alt consists of two components working together:
- **Backend (Rust Daemon)**: Watches for downloaded samples and organizes them
- **Frontend (Browser Extension)**: Captures metadata from Splice web requests

## ✨ Features

- 🔍 **Automatic Detection**: Monitors download directory for new samples
- 🗂️ **Smart Organization**: Categorizes samples using Bitwig Studio's built-in categories  
- 📊 **SQLite Database**: Stores metadata for fast searching and deduplication
- 🏷️ **Tag-Based Mapping**: Automatically maps Splice tags to appropriate categories
- 🚫 **Smart Deduplication**: Prevents duplicate samples using file hashing with existence verification
- ⚡ **Low Overhead**: Efficient and lightweight for constant background operation
- 🔄 **Error Recovery**: Robust error handling with retry mechanisms and structured logging
- 🌐 **Browser Integration**: Seamless metadata capture from Splice website
- 🎨 **User-Friendly**: Colorized terminal output and clear error messages

## 📂 Bitwig Categories

The daemon automatically maps samples to these Bitwig-compatible categories:
- Bass, Bell, Brass, Chip, Cymbal, Drone
- Drum Loop, Guitar, Hi-hat, Keyboards, Kick, Lead
- Mallet, Orchestral, Organ, Other Drums, Pad, Percussion
- Piano, Snare, Sound FX, Strings, Synth, Tom, Unknown, Vocal, Winds

## ⚡ Quick Start

**Automated Installation:**
```bash
git clone https://github.com/erelsop/splice-alt.git
cd splice-alt
./install.sh
```

**Manual Steps:**
```bash
# 1. Build the daemon
cd splice-alt/backend
cargo build --release

# 2. Package the browser extension
cd ../frontend
./package.sh

# 3. Install extension in browser (drag .zip to chrome://extensions/)

# 4. Start the daemon 
# New unified approach (recommended)
./target/release/splice-alt-daemon run --daemonize

# Or traditional background start (backward compatible)
./target/release/splice-alt-daemon start

# Or foreground mode for testing
./target/release/splice-alt-daemon run

# 5. Download samples from Splice.com - they'll be auto-organized!
```

## 🚀 Installation

### Prerequisites
- **Rust** (latest stable version)
- **Linux** system with inotify support
- **Chrome/Firefox** browser
- **zip** utility (auto-installed by install.sh)

### 1. Build the Daemon

```bash
git clone https://github.com/erelsop/splice-alt.git
cd splice-alt/backend
cargo build --release

# Optional: Copy to system PATH for easier access
sudo cp target/release/splice-alt-daemon /usr/local/bin/

# Or create an alias (add to your ~/.bashrc or ~/.zshrc)
echo 'alias splicealt="~/src/splice-alt/backend/target/release/splice-alt-daemon"' >> ~/.zshrc
```

### 2. Install Browser Extension

> **📝 Browser Compatibility Note:** Firefox has been the primary development and testing focus. Chromium-based browsers (Chrome, Edge, etc.) have had minimal testing and may have compatibility issues. Firefox is recommended for the most stable experience.

#### Option A: Pre-packaged Extension (Recommended)
1. Navigate to `splice-alt/frontend/`
2. Run `./package.sh` to create the extension package
3. Install the generated `.zip` file:

**Firefox:**
- Open `about:debugging`
- Click "This Firefox" 
- Click "Load Temporary Add-on"
- Select `manifest.json` from the extracted package

**Chromium:**
- Open `chrome://extensions/`
- Enable "Developer mode"
- Drag and drop `splice-alt-extension-v1.0.0.zip` onto the page
- OR click "Load unpacked" and select the `frontend/` directory

#### Option B: Developer Mode (Manual)

**Firefox:**
1. Open `about:debugging`
2. Click "This Firefox"
3. Click "Load Temporary Add-on"
4. Select `splice-alt/frontend/manifest.json`

**Chromium:**
1. Open `chrome://extensions/`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select the `splice-alt/frontend/` directory

## 🎮 Usage

### Basic Setup

1. **Start the Daemon**:
```bash
# New unified approach (recommended)
./target/release/splice-alt-daemon run --daemonize

# Or traditional background start (backward compatible)
./target/release/splice-alt-daemon start

# Or foreground mode with custom paths
./target/release/splice-alt-daemon run \
  --watch-dir ~/Downloads \
  --library-dir ~/Music/Samples/SpliceLib \
  --database ~/.local/share/splice-alt/samples.db
```

2. **Configure Browser Extension**:
   - Click the Splice Alt extension icon
   - Enable "Extension Enabled" and "Auto-create JSON files"
   - Optionally set your watch directory

3. **Download Samples**:
   - Browse Splice.com normally
   - Download samples - JSON metadata files will be created automatically
   - The daemon will detect and organize them into your library

### Daemon Management

#### Start Background Daemon
```bash
# New approach (recommended)
./target/release/splice-alt-daemon run --daemonize

# Traditional approach (backward compatible)
./target/release/splice-alt-daemon start
```

#### Stop Background Daemon
```bash
./target/release/splice-alt-daemon stop
```

#### Check Daemon Status
```bash
./target/release/splice-alt-daemon status
```

The daemon will:
- Run in the background without keeping a terminal open
- Log activity to `~/.cache/splice-alt-daemon.log` using structured logging
- Store its process ID in `~/.cache/splice-alt-daemon.pid`
- Provide colorized, user-friendly error messages
- Automatically restart if the system reboots (when added to startup)

**Note:** If you've set up an alias called `splicealt`, you can use shorter commands:
- `splicealt run --daemonize` or `splicealt start`
- `splicealt stop`
- `splicealt status`

### Advanced Usage

#### List Organized Samples
```bash
./target/release/splice-alt-daemon list --category Bass
```

#### Test Metadata Parsing
```bash
./target/release/splice-alt-daemon test path/to/metadata.json
```

#### Process Files Directly
```bash
./target/release/splice-alt-daemon process \
  sample.wav metadata.json \
  --library-dir ~/Music/Samples/SpliceLib \
  --database ~/samples.db
```

#### Update File Paths in Database
```bash
./target/release/splice-alt-daemon update-path old/path new/path
```

#### Debug Mode
Use `Ctrl+Shift+S` on Splice.com to toggle the debug status panel.

## 📁 Directory Structure

### Default Paths
- **Watch Directory**: `~/Downloads` (where browser downloads samples)
- **Library Directory**: `~/Music/Samples/SpliceLib`  
- **Database**: `~/.local/share/splice-alt/samples.db`
- **Log File**: `~/.cache/splice-alt-daemon.log`

### Library Organization
```
~/Music/Samples/SpliceLib/
├── Lead/
│   └── Electronic Vibes Vol. 1/
│       └── sample_lead_128.wav
├── Bass/
│   └── Deep House Pack/
│       └── bass_fundamental_Am.wav
├── Drum Loop/
│   └── Trap Essentials/
│       └── drum_loop_140_hard.wav
└── ...
```

## 🧩 Browser Extension Features

- **webRequest API Monitoring**: Captures Splice API responses at browser level
- **Automatic JSON Creation**: Creates metadata files alongside WAV downloads
- **Visual Feedback**: Shows notifications when samples are processed
- **Status Monitoring**: Real-time stats and activity tracking (Ctrl+Shift+S)
- **Smart Detection**: Only processes WAV files from Splice
- **Zero Page Interference**: No content script conflicts with Splice website

## 🔧 Configuration

### Daemon Configuration
The daemon accepts command-line arguments for all paths:

```bash
./target/release/splice-alt-daemon --help
```

### Browser Extension Settings
Access via the extension popup:
- Enable/disable automatic processing
- Configure auto-creation of JSON files
- Set watch directory (optional)

## 🐛 Troubleshooting

### Common Issues

**Samples not being organized:**
- Ensure daemon is running and watching correct directory
- Check that both WAV and JSON files appear in downloads
- Verify extension permissions for splice.com
- Check the log file at `~/.cache/splice-alt-daemon.log` for detailed error messages

**Database errors:**
- Check database directory permissions
- Ensure SQLite can write to the specified path
- Database path is automatically created if parent directories don't exist

**File permission errors:**
- Verify write permissions for library directory
- Check that daemon can access watch directory
- Daemon uses safe file operations with verification

**Daemon startup issues:**
- Use `splicealt status` to check if daemon is already running
- Check for PID file conflicts at `~/.cache/splice-alt-daemon.pid`
- Daemon includes race condition protection for concurrent starts

### Error Recovery

The daemon includes robust error handling with modern Rust practices:
- **Structured Logging**: Detailed async-safe logging with tracing framework
- **Automatic Retry**: Failed operations retry with exponential backoff
- **File Validation**: Ensures files exist and are valid before processing
- **Safe File Moving**: Copy-verify-delete pattern prevents data loss
- **Database Recovery**: Retry mechanisms for database operations
- **Smart Duplicate Handling**: Checks both database hash AND physical file existence
- **Colorized Output**: User-friendly terminal messages with console crate
- **Graceful Error Messages**: Clear error context using thiserror and anyhow

## 📊 Database Schema

The SQLite database stores comprehensive metadata:

```sql
CREATE TABLE samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL UNIQUE,
    pack_name TEXT NOT NULL,
    filename TEXT NOT NULL,
    file_hash TEXT NOT NULL UNIQUE,
    bpm INTEGER,
    audio_key TEXT,
    mapped_category TEXT NOT NULL,
    tags TEXT, -- JSON array
    date_downloaded TEXT NOT NULL,
    sample_id TEXT,
    artist_name TEXT,
    genre TEXT,
    duration_ms INTEGER,
    sample_rate INTEGER,
    bit_depth INTEGER,
    file_size INTEGER,
    download_url TEXT,
    preview_url TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_samples_hash ON samples(file_hash);
CREATE INDEX idx_samples_category ON samples(mapped_category);
CREATE INDEX idx_samples_pack ON samples(pack_name);
CREATE INDEX idx_samples_tags ON samples(tags);
```

## 🔮 Future Enhancements

- **Manual Tagging Interface**: GUI for correcting categorization
- **Advanced Filtering**: Search by BPM, key, tags, etc.

## 📈 Development Status

✅ **Production Ready**:
- Core daemon architecture with file watching
- Complete metadata parsing and validation
- Database operations with smart deduplication
- Library organization with Bitwig categories
- Browser extension with webRequest API monitoring
- Comprehensive error handling and recovery with modern Rust practices
- Safe file operations with verification
- Packaged extension for easy installation
- All CLI commands implemented and tested
- Structured async-safe logging with tracing
- Enhanced enum parsing with strum
- Colorized terminal output and user-friendly error messages

🚧 **Future Enhancements**:
- Enhanced tag mapping rules
- Configuration file support
- Performance optimizations

## 🛠️ Dependencies

### Backend (Rust)
- `notify` - File system watching
- `rusqlite` - SQLite database access
- `serde` / `serde_json` - JSON parsing
- `sha2` - File hashing for deduplication
- `tokio` - Async runtime with timeouts
- `clap` - Command line parsing
- `anyhow` - Error handling
- `tracing` / `tracing-subscriber` / `tracing-appender` - Structured async-safe logging
- `strum` - Enhanced enum parsing and serialization
- `thiserror` - Structured error types
- `console` - Colorized terminal output
- `tempfile` - Temporary files for testing

### Frontend (Browser Extension)
- Chrome Extensions Manifest V3
- Web Request API for intercepting Splice calls
- Downloads API for automatic file creation
- Storage API for configuration

## 🤝 Contributing

This project welcomes contributions! Areas for improvement:

- Additional music categorization rules
- Support for other sample services
- GUI management interface
- Performance optimizations
- Documentation improvements

## 📄 License

This project is dedicated to the public domain under the [CC0 1.0 Universal License](LICENSE).

## Disclaimer

**This project is not affiliated with, endorsed by, or connected to Splice in any official way. It is a personal utility that helps users locally organize sample files they have legally downloaded via their own Splice accounts. This tool does not access, modify, or interact with Splice systems beyond observing client-side metadata and file downloads already authorized to the user. No Splice content is redistributed. Use of this tool assumes that the user adheres to Splice's Terms of Use.**

---

**Happy sample organizing! 🎵**
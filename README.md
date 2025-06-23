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
- 🚫 **Deduplication**: Prevents duplicate samples using file hashing
- ⚡ **Low Overhead**: Efficient and lightweight for constant background operation
- 🔄 **Error Recovery**: Robust error handling with retry mechanisms
- 🌐 **Browser Integration**: Seamless metadata capture from Splice website

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
# 4. Start the daemon (background mode recommended)
# Background mode (recommended)
splice-alt-daemon --start

# Or foreground mode
cd ../backend
./target/release/splice-alt-daemon watch

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

# Optional: Copy to system PATH
sudo cp target/release/splice-alt-daemon /usr/local/bin/
```

### 2. Install Browser Extension

#### Option A: Pre-packaged Extension (Recommended)
1. Navigate to `splice-alt/frontend/`
2. Run `./package.sh` to create the extension package
3. Install the generated `.zip` file:

**Chrome/Chromium/Edge:**
- Open `chrome://extensions/`
- Enable "Developer mode"
- Drag and drop `splice-alt-extension-v1.0.0.zip` onto the page
- OR click "Load unpacked" and select the `frontend/` directory

**Firefox:**
- Open `about:debugging`
- Click "This Firefox" 
- Click "Load Temporary Add-on"
- Select `manifest.json` from the extracted package

#### Option B: Developer Mode (Manual)
**Chrome/Chromium:**
1. Open `chrome://extensions/`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select the `splice-alt/frontend/` directory

**Firefox:**
1. Open `about:debugging`
2. Click "This Firefox"
3. Click "Load Temporary Add-on"
4. Select `splice-alt/frontend/manifest.json`

## 🎮 Usage

### Basic Setup

1. **Start the Daemon**:
# Use default directories (watches ~/Downloads)
./target/release/splice-alt-daemon watch

# Or specify custom paths
./target/release/splice-alt-daemon watch \
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
splice-alt-daemon --start
```

#### Stop Background Daemon
```bash
splice-alt-daemon --stop
```

#### Check Daemon Status
```bash
splice-alt-daemon --status
```

The daemon will:
- Run in the background without keeping a terminal open
- Log activity to `~/.cache/splice-alt-daemon.log`
- Store its process ID in `~/.cache/splice-alt-daemon.pid`
- Automatically restart if the system reboots (when added to startup)


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

**Database errors:**
- Check database directory permissions
- Ensure SQLite can write to the specified path

**File permission errors:**
- Verify write permissions for library directory
- Check that daemon can access watch directory

### Error Recovery

The daemon includes robust error handling:
- **Automatic Retry**: Failed operations retry with exponential backoff
- **File Validation**: Ensures files exist and are valid before processing
- **Safe File Moving**: Copy-verify-delete pattern prevents data loss
- **Database Recovery**: Retry mechanisms for database operations
- **Duplicate Handling**: Automatic cleanup of duplicate files

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
- Database operations with deduplication
- Library organization with Bitwig categories
- Browser extension with webRequest API monitoring
- Comprehensive error handling and recovery
- Safe file operations with verification
- Packaged extension for easy installation
- All CLI commands implemented and tested

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

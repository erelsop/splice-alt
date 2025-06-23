# Splice Alt Browser Extension

A browser extension that automatically captures metadata from Splice.com and creates JSON files alongside sample downloads for the Splice Alt daemon to process.

## 🚀 Quick Install

**Production Ready Package:**
1. Run `./package.sh` to create `splice-alt-extension-v1.0.0.zip`
2. Install the package:
   - **Chrome/Edge**: Drag `.zip` file to `chrome://extensions/` (Developer mode on)
   - **Firefox**: Extract and load `manifest.json` from `about:debugging`

## ✨ Features

- **webRequest API Monitoring**: Captures Splice API responses at browser level
- **Automatic JSON Creation**: Creates metadata files alongside WAV downloads
- **Visual Feedback**: Shows notifications when samples are processed
- **Zero Page Interference**: No conflicts with Splice website functionality
- **Debug Panel**: Access debug information with `Ctrl+Shift+S` on Splice.com
- **Configurable Settings**: Enable/disable features through popup interface

## 📦 Installation Options

### Option A: Pre-packaged (Recommended)
```bash
./package.sh  # Creates splice-alt-extension-v1.0.0.zip
```

**Chrome/Chromium/Edge:**
- Open `chrome://extensions/`
- Enable "Developer mode"
- Drag and drop the `.zip` file onto the page

**Firefox:**
- Open `about:debugging`
- Click "This Firefox" → "Load Temporary Add-on"
- Select `manifest.json` from extracted package

### Option B: Developer Mode
**Chrome/Chromium/Edge:**
1. Open `chrome://extensions/`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select this directory (`frontend/`)

**Firefox:**
1. Open `about:debugging`
2. Click "This Firefox" → "Load Temporary Add-on"
3. Select `manifest.json` from this directory

## 🎮 Usage

1. Install the extension using one of the methods above
2. Navigate to Splice.com and browse samples
3. Download samples normally - JSON metadata files are created automatically
4. The Splice Alt daemon detects and organizes samples with their metadata
5. Find organized samples in `~/Music/Samples/SpliceLib/`

## ⚙️ Configuration

Click the extension icon in your browser toolbar to access settings:
- **Extension Enabled**: Toggle the extension on/off
- **Auto-create JSON files**: Automatically create metadata files (recommended)
- **Watch Directory**: Optional override for where files are saved

## 🐛 Debug Features

- Press `Ctrl+Shift+S` on Splice.com to toggle the debug status panel
- View real-time API interception status and statistics
- Monitor extension activity and troubleshoot issues

## 🔧 Technical Details

The extension uses the browser's webRequest API to monitor network traffic and capture Splice API responses. When a sample is downloaded, it extracts the metadata from the API response and creates a corresponding JSON file.

**Architecture:**
- **Background Script**: Monitors webRequest API for Splice API calls
- **Content Script**: Provides visual feedback and debug interface
- **Popup Interface**: Settings and statistics management

### Supported Browsers
- Chrome 88+ (Manifest V3)
- Firefox 78+ (Manifest V2 compatibility)
- Edge 88+
- Other Chromium-based browsers

### Required Permissions
- `webRequest`/`webRequestBlocking`: Monitor Splice API responses
- `downloads`: Create JSON metadata files
- `storage`: Save extension settings
- `activeTab`: Content script injection for feedback
- `https://splice.com/*`: Access to Splice website

## 📊 Metadata Captured

The extension captures complete sample metadata including:
- Sample filename, file hash, pack name
- BPM, musical key, tags
- Artist information, genre classification
- Download credits and purchase information
- All fields needed for intelligent categorization

## 🎯 Production Ready

This extension is fully functional and ready for daily use:
- ✅ Robust error handling with retry mechanisms  
- ✅ No unauthorized API requests (observes existing traffic)
- ✅ Compatible with Firefox and Chrome
- ✅ Packaged for easy distribution
- ✅ Zero interference with Splice website functionality

## Troubleshooting

### Extension Not Working
1. Check that the extension is enabled in the popup
2. Make sure you're on splice.com
3. Verify webRequest permissions are granted
4. Check the browser console for errors

### No JSON Files Created
1. Verify that "Auto-create JSON files" is enabled
2. Make sure you're downloading samples (not just previewing)
3. Check that the extension has permission to access downloads
4. Look for notifications showing metadata capture

### Debug Information
- Press `Ctrl+Shift+S` on Splice.com to toggle the debug panel
- Check the popup for statistics on captured metadata
- Use the browser's developer tools to check console logs

## File Structure

When working correctly, you should see:
```
~/Downloads/
├── sample_name.wav          # Downloaded from Splice
└── sample_name.json         # Created by extension
```

The daemon will then process both files and organize them into your library at `~/Music/Samples/SpliceLib/`. 
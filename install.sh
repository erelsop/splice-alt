#!/bin/bash

# Splice Alt - Complete System Installation Script
# Builds daemon, packages extension, and sets up the complete system

set -e

echo "🎵 Splice Alt - Automatic Sample Library Organizer"
echo "=================================================="
echo ""

# Validate we're in the right directory
if [[ ! -d "backend" || ! -d "frontend" || ! -f "backend/Cargo.toml" ]]; then
    echo "❌ Error: This script must be run from the splice-alt project root directory"
    echo "   Expected to find: backend/, frontend/, and backend/Cargo.toml"
    echo "   Current directory: $(pwd)"
    exit 1
fi

# Check prerequisites
echo "🔍 Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "   source ~/.cargo/env"
    exit 1
fi

# Check Rust version
RUST_VERSION=$(cargo --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
echo "✅ Found Rust/Cargo version: $RUST_VERSION"

if ! command -v zip &> /dev/null; then
    echo "📦 Installing zip utility..."
    
    # Try different package managers
    if command -v apt &> /dev/null; then
        echo "   Using apt (Debian/Ubuntu)..."
        sudo apt update && sudo apt install -y zip
    elif command -v yum &> /dev/null; then
        echo "   Using yum (RHEL/CentOS)..."
        sudo yum install -y zip
    elif command -v dnf &> /dev/null; then
        echo "   Using dnf (Fedora)..."
        sudo dnf install -y zip
    elif command -v pacman &> /dev/null; then
        echo "   Using pacman (Arch)..."
        sudo pacman -S --noconfirm zip
    elif command -v apk &> /dev/null; then
        echo "   Using apk (Alpine)..."
        sudo apk add zip
    elif command -v zypper &> /dev/null; then
        echo "   Using zypper (openSUSE)..."
        sudo zypper install -y zip
    else
        echo "❌ Could not detect package manager. Please install 'zip' utility manually:"
        echo "   - Debian/Ubuntu: sudo apt install zip"
        echo "   - RHEL/CentOS: sudo yum install zip"
        echo "   - Fedora: sudo dnf install zip"
        echo "   - Arch: sudo pacman -S zip"
        echo "   - Alpine: sudo apk add zip"
        exit 1
    fi
    
    # Verify zip was installed
    if ! command -v zip &> /dev/null; then
        echo "❌ Failed to install zip utility"
        exit 1
    fi
fi

echo "✅ Prerequisites satisfied"
echo ""

# Build daemon
echo "🔨 Building Rust daemon..."
if ! cd backend; then
    echo "❌ Failed to enter backend directory"
    exit 1
fi

if ! cargo build --release; then
    echo "❌ Failed to build Rust daemon"
    echo "   Try running: cargo clean && cargo build --release"
    exit 1
fi

# Verify the binary was created and is executable
DAEMON_PATH="target/release/splice-alt-daemon"
if [[ ! -f "$DAEMON_PATH" ]]; then
    echo "❌ Daemon binary not found at $DAEMON_PATH"
    exit 1
fi

if [[ ! -x "$DAEMON_PATH" ]]; then
    echo "❌ Daemon binary is not executable"
    exit 1
fi

echo "✅ Daemon built successfully: $(pwd)/$DAEMON_PATH"
echo ""

# Package extension
echo "📦 Packaging browser extension..."
if ! cd ../frontend; then
    echo "❌ Failed to enter frontend directory"
    exit 1
fi

if [[ ! -f "package.sh" ]]; then
    echo "❌ package.sh not found in frontend directory"
    exit 1
fi

chmod +x package.sh

if ! ./package.sh; then
    echo "❌ Failed to package browser extension"
    exit 1
fi

# Verify the package was created
if [[ ! -f "splice-alt-extension-v1.0.0.zip" ]]; then
    echo "❌ Extension package not found"
    exit 1
fi

echo "✅ Extension packaged successfully"
echo ""

# Return to project root
cd ..

# Optional system installation
echo "🚀 Installation Options:"
echo ""
echo "1. Install daemon system-wide (recommended):"
echo "   sudo cp backend/target/release/splice-alt-daemon /usr/local/bin/"
echo ""
echo "2. Browser extension package created:"
echo "   frontend/splice-alt-extension-v1.0.0.zip"
echo ""
echo "3. Install browser extension:"
echo "   - Chrome: Drag .zip to chrome://extensions/ (with Developer mode on)"
echo "   - Firefox: Load manifest.json from about:debugging"
echo ""

read -p "Install daemon to /usr/local/bin? (y/N): " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    if sudo cp backend/target/release/splice-alt-daemon /usr/local/bin/; then
        echo "✅ Daemon installed to /usr/local/bin/splice-alt-daemon"
        
        # Verify installation
        if command -v splice-alt-daemon &> /dev/null; then
            echo "✅ Daemon is now available in PATH"
        else
            echo "⚠️  Daemon installed but not found in PATH. You may need to restart your shell."
        fi
    else
        echo "❌ Failed to install daemon to /usr/local/bin/"
        echo "   You can run it directly from: $(pwd)/backend/target/release/splice-alt-daemon"
    fi
else
    echo "ℹ️  Daemon available at: $(pwd)/backend/target/release/splice-alt-daemon"
fi

echo ""
echo "🎯 Installation Complete!"
echo ""
echo "Next steps:"
echo "1. Install browser extension using the .zip file"
if command -v splice-alt-daemon &> /dev/null; then
    echo "2. Start daemon: splice-alt-daemon watch"
else
    echo "2. Start daemon: ./backend/target/release/splice-alt-daemon watch"
fi
echo "3. Download samples from Splice.com"
echo "4. Find organized samples in ~/Music/Samples/SpliceLib/"
echo ""
if command -v splice-alt-daemon &> /dev/null; then
    echo "For help: splice-alt-daemon --help"
else
    echo "For help: ./backend/target/release/splice-alt-daemon --help"
fi
echo "Debug panel: Ctrl+Shift+S on Splice.com"
echo ""
echo "Happy sample organizing! 🎵" 
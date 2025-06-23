#!/bin/bash

# Splice Alt Extension Packaging Script
# Creates a distributable .zip file for the browser extension

set -e

EXTENSION_NAME="splice-alt-extension"
VERSION=$(grep '"version"' manifest.json | sed 's/.*"version": "\([^"]*\)".*/\1/')
PACKAGE_NAME="${EXTENSION_NAME}-v${VERSION}.zip"

echo "📦 Packaging Splice Alt Extension v${VERSION}..."

# Clean previous packages
rm -f *.zip

# Create package with only necessary files
zip -r "${PACKAGE_NAME}" \
    manifest.json \
    background.js \
    content.js \
    popup.html \
    popup.js \
    -x "*.sh" "README.md" "*.md"

echo "✅ Package created: ${PACKAGE_NAME}"
echo ""
echo "📋 Installation Instructions:"
echo "Chrome/Edge:"
echo "  1. Open chrome://extensions/"
echo "  2. Enable 'Developer mode'"
echo "  3. Click 'Load unpacked' and select this directory"
echo "  OR"
echo "  4. Drag and drop ${PACKAGE_NAME} onto the extensions page"
echo ""
echo "Firefox:"
echo "  1. Open about:debugging"
echo "  2. Click 'This Firefox'"
echo "  3. Click 'Load Temporary Add-on'"
echo "  4. Select manifest.json from extracted ${PACKAGE_NAME}"
echo ""
echo "🎯 Ready to distribute!" 
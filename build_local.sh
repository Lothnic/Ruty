#!/bin/bash
set -e

echo "🚀 Building Ruty for Linux..."

# Check requirements
if ! command -v uv &> /dev/null; then
    echo "❌ 'uv' not found. Please install it."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ 'cargo' not found. Please install Rust."
    exit 1
fi

# 1. Install dependencies
echo "📦 Installing Python dependencies..."
uv sync
uv pip install pyinstaller

# 2. Build Python Sidecar
echo "🐍 Building Python sidecar..."
# Tauri expects the binary name to include the target triple
# For Linux x86_64, this is typically x86_64-unknown-linux-gnu
TARGET="x86_64-unknown-linux-gnu"
mkdir -p src-tauri/binaries

echo "   Target: $TARGET"
uv run pyinstaller --clean --noconfirm --log-level WARN \
    --onefile \
    --name ruty-backend-$TARGET \
    --collect-all ruty \
    ruty/server.py \
    --distpath src-tauri/binaries

echo "✅ Sidecar built at src-tauri/binaries/ruty-backend-$TARGET"

# 3. Build Tauri
echo "🦀 Building Tauri app (.deb)..."
cd src-tauri

# WORKAROUND: Ubuntu 24.04 uses ayatana-appindicator but some tools look for appindicator3
# We create a local alias for pkg-config
if pkg-config --exists ayatana-appindicator3-0.1 && ! pkg-config --exists appindicator3-0.1; then
    echo "🔧 Applying pkg-config workaround for Ubuntu 24.04..."
    mkdir -p pkgconfig
    # Copy/Symlink the pc file
    cp $(pkg-config --variable=pcfiledir ayatana-appindicator3-0.1)/ayatana-appindicator3-0.1.pc pkgconfig/appindicator3-0.1.pc
    # Adjust content if necessary (usually just a name alias works if libs match, or we just point PKG_CONFIG_PATH)
    export PKG_CONFIG_PATH="$(pwd)/pkgconfig:$PKG_CONFIG_PATH"
    echo "   Set PKG_CONFIG_PATH to include local alias."
fi

# Install/Update tauri-cli to Ensure v2
echo "   Ensuring tauri-cli v2 is installed..."
cargo install tauri-cli --version "^2.0.0" --locked

cargo tauri build

echo "🎉 Build Complete!"
echo "   Artifacts located in: src-tauri/target/release/bundle/deb/"

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

# Install tauri-cli if missing
if ! cargo tauri --version &> /dev/null; then
    echo "   'cargo tauri' not found. Installing tauri-cli..."
    cargo install tauri-cli
fi

cargo tauri build

echo "🎉 Build Complete!"
echo "   Artifacts located in: src-tauri/target/release/bundle/deb/"

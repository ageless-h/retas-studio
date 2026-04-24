#!/bin/bash
set -e

echo "=== RETAS Studio Development Environment Setup ==="

if command -v rustc &> /dev/null; then
    echo "Rust is already installed: $(rustc --version)"
else
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "Rust installed: $(rustc --version)"
fi

if ! command -v cargo &> /dev/null; then
    echo "Cargo not found. Please restart your terminal or run: source \$HOME/.cargo/env"
    exit 1
fi
echo "Cargo is available: $(cargo --version)"

echo ""
echo "Installing Rust components..."
rustup component add clippy rustfmt rust-analyzer

echo ""
echo "Checking workspace compilation..."
cargo check --workspace

echo ""
echo "=== Backend Ready ==="

if command -v node &> /dev/null; then
    echo "Node.js is available: $(node --version)"
else
    echo "Node.js not found. Install Node.js 18+ for frontend development."
    echo "  https://nodejs.org/"
fi

echo ""
echo "=== Setup Complete ==="
echo "To start development:"
echo "  cd retas-tauri"
echo "  npm install"
echo "  npm run tauri dev"

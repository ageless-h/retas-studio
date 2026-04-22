#!/bin/bash

set -e

echo "=== RETAS Studio Development Environment Setup ==="

if command -v rustc &> /dev/null; then
    echo "✓ Rust is already installed: $(rustc --version)"
else
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✓ Rust installed: $(rustc --version)"
fi

if command -v cargo &> /dev/null; then
    echo "✓ Cargo is available: $(cargo --version)"
else
    echo "✗ Cargo not found. Please restart your terminal or run: source \$HOME/.cargo/env"
    exit 1
fi

echo ""
echo "Installing Rust components..."
rustup component add clippy rustfmt rust-analyzer

echo ""
echo "Installing common tools..."
cargo install cargo-watch cargo-edit

cd "$(dirname "$0")/retas-studio"

echo ""
echo "Checking project compilation..."
cargo check

echo ""
echo "=== Setup Complete ==="
echo "To start development:"
echo "  cd retas-studio"
echo "  cargo run"

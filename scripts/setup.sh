#!/usr/bin/env bash
set -e

echo "Checking Rust installation..."

if ! command -v rustc &> /dev/null; then
  echo "Rust not found. Installing rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
else
  echo "Rust already installed."
fi

echo "Installing toolchain components..."

rustup component add rustfmt clippy

echo "Setup complete."
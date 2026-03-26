#!/bin/bash
# Profile Spout: build with profiling, run the game, then open puffin_viewer
# with the saved data.
#
# Usage: ./profile.sh
#
# Play the game, then quit (Escape). A profile.puffin file is saved on exit.
# puffin_viewer then opens the saved file so you can explore the data.
#
# Prerequisites:
#   cargo install puffin_viewer

set -euo pipefail

# Ensure puffin_viewer is installed.
if ! command -v puffin_viewer &>/dev/null; then
    echo "puffin_viewer not found. Installing..."
    cargo install puffin_viewer
fi

echo "Building with profiling..."
cargo build --features profiling --release 2>&1

echo "Starting game... (quit with Escape when done)"
./target/release/spout || true

if [ ! -f profile.puffin ]; then
    echo "Error: profile.puffin was not created."
    exit 1
fi

echo "Opening profiling data in puffin_viewer..."
puffin_viewer profile.puffin

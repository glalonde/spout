#!/bin/bash
# Profile Spout: build with profiling, run the game with puffin_viewer
# connected for live profiling.
#
# Usage: ./profile.sh
#
# puffin_viewer connects to the game's puffin_http server and shows
# live CPU + GPU profiling data. When you quit the game (Escape),
# the viewer stays open with the accumulated data.
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

echo "Starting game..."
./target/release/spout &
GAME_PID=$!

# Wait for the puffin HTTP server to start accepting connections.
echo "Waiting for puffin server..."
for i in $(seq 1 20); do
    if nc -z 127.0.0.1 8585 2>/dev/null; then
        break
    fi
    sleep 0.25
done

echo "Starting puffin_viewer (connects to 127.0.0.1:8585)..."
puffin_viewer --url 127.0.0.1:8585 &
VIEWER_PID=$!

# Wait for the game to exit (user quits with Escape).
wait $GAME_PID 2>/dev/null || true
echo ""
echo "Game exited. puffin_viewer is still open with the profiling data."
echo "Close the viewer window when done."
wait $VIEWER_PID 2>/dev/null || true

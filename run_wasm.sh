#!/usr/bin/env bash

# Pulled from https://github.com/gfx-rs/wgpu/blob/master/run-wasm-example.sh

set -ex

echo "Compiling..."
RUSTFLAGS=--cfg=web_sys_unstable_apis
cargo build --target wasm32-unknown-unknown --features "$1"

# Use the following for an example(as opposed to the main binary):
# cargo build --example $1 --target wasm32-unknown-unknown --features "$2"

TITLE="spout"
OUTPUT_DIR="target/wasm-examples/$TITLE"

echo "Generating bindings..."
mkdir -p target/wasm-binary/$TITLE
wasm-bindgen --target web --out-dir target/wasm-examples/$TITLE target/wasm32-unknown-unknown/debug/$TITLE.wasm
cat wasm-resources/index.template.html | sed "s/{{example}}/$TITLE/g" > target/wasm-examples/$TITLE/index.html

# Find a serving tool to host the example
SERVE_CMD=""
SERVE_ARGS=""
if which basic-http-server; then
    SERVE_CMD="basic-http-server"
    SERVE_ARGS="target/wasm-examples/$TITLE -a 127.0.0.1:1234"
elif which miniserve && python3 -m http.server --help > /dev/null; then
    SERVE_CMD="miniserve"
    SERVE_ARGS="target/wasm-examples/$TITLE -p 1234 --index index.html"
elif python3 -m http.server --help > /dev/null; then
    SERVE_CMD="python3"
    SERVE_ARGS="-m http.server --directory target/wasm-examples/$TITLE 1234"
fi

# Exit if we couldn't find a tool to serve the example with
if [ "$SERVE_CMD" = "" ]; then
    echo "Couldn't find a utility to use to serve the example web page. You can serve the `target/wasm-examples/$TITLE` folder yourself using any simple static http file server."
fi

echo "Serving example with $SERVE_CMD at http://localhost:1234"
$SERVE_CMD $SERVE_ARGS
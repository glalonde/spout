#!/usr/bin/env bash

# Pulled from https://github.com/gfx-rs/wgpu/blob/master/run-wasm-example.sh

set -ex

echo "Compiling..."
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --no-default-features --target wasm32-unknown-unknown

TITLE="spout"
OUTPUT_DIR="target/wasm-examples/$TITLE"

echo "Generating bindings..."
mkdir -p target/wasm-binary/$TITLE
wasm-bindgen --target web --out-dir $OUTPUT_DIR target/wasm32-unknown-unknown/debug/$TITLE.wasm
cat wasm-resources/index.template.html | sed "s/{{example}}/$TITLE/g" > $OUTPUT_DIR/index.html

# Find a serving tool to host the example
SERVE_CMD=""
SERVE_ARGS=""
if which basic-http-server; then
    SERVE_CMD="basic-http-server"
    SERVE_ARGS="$OUTPUT_DIR -a 127.0.0.1:1234"
elif which miniserve && python3 -m http.server --help > /dev/null; then
    SERVE_CMD="miniserve"
    SERVE_ARGS="$OUTPUT_DIR -p 1234 --index index.html"
elif python3 -m http.server --help > /dev/null; then
    SERVE_CMD="python3"
    SERVE_ARGS="-m http.server --directory $OUTPUT_DIR 1234"
fi

# Exit if we couldn't find a tool to serve the example with
if [ "$SERVE_CMD" = "" ]; then
    echo "Couldn't find a utility to use to serve the example web page. You can serve the `$OUTPUT_DIR` folder yourself using any simple static http file server."
fi

echo "Serving example with $SERVE_CMD at http://localhost:1234"
$SERVE_CMD $SERVE_ARGS
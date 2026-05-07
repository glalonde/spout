#!/usr/bin/env bash

# Pulled from https://github.com/gfx-rs/wgpu/blob/master/run-wasm-example.sh
#
# Usage:
#   ./run_wasm.sh             - Serve plain HTTP, bound to all interfaces.
#   ./run_wasm.sh --https     - Serve HTTPS via Tailscale (required for iOS
#                                Safari WebGPU on a non-localhost host).
#
# In --https mode the dev server stays on 127.0.0.1 and `tailscale serve` is
# run in the foreground as the public-facing endpoint. Ctrl+C exits both.

set -e

USE_HTTPS=0
if [ "${1:-}" = "--https" ]; then
    USE_HTTPS=1
fi

set -x

echo "Compiling..."
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --release --no-default-features --target wasm32-unknown-unknown

TITLE="spout"
OUTPUT_DIR="target/wasm-examples/$TITLE"

echo "Generating bindings..."
mkdir -p target/wasm-binary/$TITLE
wasm-bindgen --target web --out-dir $OUTPUT_DIR target/wasm32-unknown-unknown/release/$TITLE.wasm
cat wasm-resources/index.template.html | sed "s/{{example}}/$TITLE/g" > $OUTPUT_DIR/index.html

# Kill any existing server on port 1234
lsof -ti :1234 | xargs kill 2>/dev/null || true

# In HTTPS mode the dev server is reached via `tailscale serve`, so the
# underlying server only needs to listen on loopback. In plain HTTP mode we
# bind to all interfaces so other devices on Wi-Fi or Tailscale can reach it.
if [ "$USE_HTTPS" = "1" ]; then
    BIND_ADDR="127.0.0.1:1234"
    BIND_HOST="127.0.0.1"
else
    BIND_ADDR="0.0.0.0:1234"
    BIND_HOST="0.0.0.0"
fi

# Find a serving tool to host the example
SERVE_CMD=""
SERVE_ARGS=""
if which basic-http-server; then
    SERVE_CMD="basic-http-server"
    SERVE_ARGS="$OUTPUT_DIR -a $BIND_ADDR"
elif which miniserve && python3 -m http.server --help > /dev/null; then
    SERVE_CMD="miniserve"
    SERVE_ARGS="$OUTPUT_DIR -p 1234 -i $BIND_HOST --index index.html"
elif python3 -m http.server --help > /dev/null; then
    SERVE_CMD="python3"
    SERVE_ARGS="-m http.server --directory $OUTPUT_DIR --bind $BIND_HOST 1234"
fi

if [ "$SERVE_CMD" = "" ]; then
    echo "Couldn't find a utility to use to serve the example web page. You can serve the $OUTPUT_DIR folder yourself using any simple static http file server."
    exit 1
fi

if [ "$USE_HTTPS" = "1" ]; then
    if ! command -v tailscale >/dev/null 2>&1; then
        echo "tailscale CLI not found in PATH; install Tailscale or run without --https." >&2
        exit 1
    fi
    TS_HOST=$(tailscale status --json 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['Self'].get('DNSName','').rstrip('.'))" 2>/dev/null || true)
    if [ -z "$TS_HOST" ]; then
        echo "Could not read Tailscale hostname; is the daemon running?" >&2
        exit 1
    fi

    set +x
    # If a previous run (especially with --bg) left an HTTPS:443 serve config
    # behind, the foreground `tailscale serve` below refuses to start. Clear
    # only the :443 binding — leaves any other serve config (other ports,
    # http://, tcp://) untouched.
    tailscale serve --https=443 off >/dev/null 2>&1 || true

    echo "Starting dev server on http://$BIND_ADDR ..."
    $SERVE_CMD $SERVE_ARGS &
    SERVER_PID=$!
    trap 'kill $SERVER_PID 2>/dev/null || true; tailscale serve --https=443 off >/dev/null 2>&1 || true' EXIT INT TERM

    # Give the dev server a moment to bind before fronting it.
    sleep 1

    echo "Fronting with Tailscale HTTPS at https://$TS_HOST/"
    echo "(Ctrl+C to stop both the dev server and the Tailscale serve proxy.)"
    set -x
    tailscale serve --https=443 http://127.0.0.1:1234
else
    set +x
    TS_URL=""
    if command -v tailscale >/dev/null 2>&1; then
        TS_IP=$(tailscale ip -4 2>/dev/null | head -n1 || true)
        if [ -n "$TS_IP" ]; then
            TS_URL="http://$TS_IP:1234"
        fi
    fi
    echo "Serving $OUTPUT_DIR with $SERVE_CMD on http://localhost:1234"
    if [ -n "$TS_URL" ]; then
        echo "  Tailscale: $TS_URL  (WebGPU may refuse without HTTPS — re-run with --https for that)"
    fi
    set -x
    $SERVE_CMD $SERVE_ARGS
fi

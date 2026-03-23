# WASM Debugging Guide

## Capturing browser console output

A headless Playwright script is at `wasm-resources/capture_console.js`. It opens
the running dev server and prints all `console.*` output and uncaught errors to
stdout, where Claude can read them.

### Setup (one-time)

```bash
npm install --save-dev playwright
npx playwright install chromium
```

### Workflow

1. Start the dev server in the background:
   ```bash
   bash run_wasm.sh &
   # wait for "Serving example..." line
   ```

2. Capture console output (default: wait 10 s):
   ```bash
   node wasm-resources/capture_console.js
   # or with custom URL / timeout:
   node wasm-resources/capture_console.js http://localhost:1234 15000
   ```

3. The script prints lines like:
   ```
   [LOG  ] wgpu: ...
   [ERROR] panicked at ...
   [PAGEERROR] RuntimeError: unreachable
   ```

4. **Expected / benign output** to ignore:
   - `Using exceptions for control flow, don't mind me` — winit intentionally
     throws a JS exception to exit the WASM event loop; harmless.
   - `No available adapters` + GPU panic — headless Chromium has no WebGPU.
     The script is useful for catching pre-GPU panics (bad init, missing APIs,
     etc.) but cannot test rendering. Use a real browser for GPU functionality.

5. Kill the server when done:
   ```bash
   kill %1   # or pkill -f "python3 -m http.server"
   ```

---

## Known WASM gotchas

### `std::time::Instant` not available
`std::time::Instant::now()` panics on `wasm32-unknown-unknown`:
```
panicked at .../std/sys/pal/wasm/../unsupported/time.rs: time not implemented on this platform
```
**Fix:** use `web_time::Instant` (drop-in replacement; backed by `performance.now()` on WASM).
`web-time` is already a direct dependency.

### `pollster::block_on` deadlock on WASM
`request_adapter` / `request_device` are JS Promises. `block_on` spins waiting
for them but never yields control to the JS event loop, so they never resolve.
**Fix:** use `wasm_bindgen_futures::spawn_local` and share results via
`Rc<RefCell<Option<T>>>`. See `examples/framework.rs` for the pattern.

### `getrandom` on WASM
wgpu 29 transitively requires `getrandom 0.3`, which needs the `wasm_js` feature
on WASM or it panics at runtime.
**Fix:** add to `Cargo.toml`:
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.3", features = ["wasm_js"] }
```

### CI: `wasm-bindgen-cli` version must match `Cargo.lock`
The CLI version must exactly match the `wasm-bindgen` crate version pinned in
`Cargo.lock`. Check with:
```bash
grep -A2 'name = "wasm-bindgen"' Cargo.lock | head -6
```
Then pin in CI: `cargo install wasm-bindgen-cli --version <exact> --locked`

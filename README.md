Spout
=====
![](https://github.com/glalonde/spout/workflows/CI/badge.svg)

Work on making a new version of a game called Spout, which I first encountered as a homebrew NDS game: https://pdroms.de/files/nintendods/spout-extended-v1-0-final

Live streaming some of the programming ony my youtube channel: https://www.youtube.com/channel/UCC_zhLitvZ0EXdZ7rhAMTOw

Linux x86_64 binaries are available on the releases page. Just set the executable bit and try it out.
```
wget https://github.com/glalonde/spout/releases/download/v0.3/spout
chmod +x ./spout
./spout
```

---

Rust Version
===

In the spirit of never finishing anything, I'm restarting in Rust. This is mostly because Cargo is easier but also because portable graphics libraries in Rust seem to be going places fast. So I'm basing this version on [wgpu-rs](https://github.com/gfx-rs/wgpu-rs). This currently runs on linux and macOS. You can try running it with `cargo run --release`. You may need to install a few depedencies, but hopefully it's not too bad.

[![Watch the video](https://img.youtube.com/vi/y-pyzTXWXds/maxresdefault.jpg)](https://youtu.be/y-pyzTXWXds)

---
Here's the final version of the previous incarnation(click for video). I'm still getting back to this point with the code in this repo, but it is on a far better foundation.

[![Watch the video](https://img.youtube.com/vi/ByFWa8JPO0c/maxresdefault.jpg)](https://youtu.be/ByFWa8JPO0c)


---

Android Dev Notes
===

Get rust logging out of ADB:
```
adb logcat RustStdoutStderr:D *:S
```

Install and run on plugged in device:
```
cargo apk run --example spout_android
```
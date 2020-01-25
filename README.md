Spout
=====

Work on making a new version of a game called Spout, which I first encountered as a homebrew NDS game: https://pdroms.de/files/nintendods/spout-extended-v1-0-final

Live streaming some of the programming ony my youtube channel: https://www.youtube.com/channel/UCC_zhLitvZ0EXdZ7rhAMTOw

---

C++ Version
===

Required deps depends on which targets you're building. At a minimum you'll need `bazel` and `clang` set up for C++17. After that, `SDL2` or `GLFW` for some parts. OpenGL, and Vulkan for other parts. gperftools, pprof.

---

Rust Version
===

In the spirit of never finishing anything, I also started implementing a Rust version. Nominally because I'm tired of trying to figure out portable C++ build systems. Rust's Cargo seems to be a bit easier to wrangle from multiple targets, with easier packaging. Also a dedicated set of developers working on making Vulkan-like interfaces more accessible, without quite as much verbosity. We'll see how it goes. Currently, I have a basic compute stage tied to a basic graphics stage, so it's working end-to-end, which is further than I got with raw Vulkan APIs. Now I just need to add a few features to bring it parity with the OpenGL version.

---
* `archaeology` has some old versions that probably don't really compile or do anything anymore.
* `base` has some generic utilities
* `src` has the SDL based version with CPU physics and OpenGL graphics. Currently the closest to an actual game.
* `graphics` has some attempt at modular OpenGL graphics.
* `gpu_particles` has an implementation of physics with OpenGL, but isn't really a playable game.
* `vulkan` has some vulkan tutorials implemented. A graphics pipeline, and a compute pipeline.
---
Here's the final version of the previous incarnation(click for video). I'm still getting back to this point with the code in this repo, but it is on a far better foundation.

[![Watch the video](https://img.youtube.com/vi/ByFWa8JPO0c/maxresdefault.jpg)](https://youtu.be/ByFWa8JPO0c)

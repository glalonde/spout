#!/bin/bash -ex
cargo build --release
perf record --call-graph dwarf,16384 -e cpu-clock -F 997 target/release/spout
perf script | /home/glalonde/git/FlameGraph/stackcollapse-perf.pl | /home/glalonde/git/FlameGraph/stackcollapse-recursive.pl | c++filt | /home/glalonde/git/FlameGraph/flamegraph.pl > flame.svg

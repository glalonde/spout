#!/bin/bash
tmp_dir=$(mktemp -d -t ci-XXXXXXXXXX)
cp /home/glalonde/git/spout/target/debug/apk/examples/Spout.apk $tmp_dir
cd $tmp_dir
unzip Spout.apk
/opt/android-sdk/build-tools/30.0.2/aapt2 d xmltree Spout.apk --file AndroidManifest.xml

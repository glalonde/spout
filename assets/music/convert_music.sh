#!/bin/bash
output_dir="output"
mkdir $output_dir
regex=".*\.(xm|s3m|mod)"
raw_files=$(find . -regextype posix-extended -iregex $regex)
for i in $raw_files; do 
  filename=$(basename $i)
  file=${filename%.*}
  # For FLAC lossless:
  # ffmpeg -i $i -c:a flac $output_dir/$file.flac
  # For vorbis VBR:
  ffmpeg -i $i -c:a libvorbis -qscale:a 5 $output_dir/$file.ogg
done

#!/bin/bash
output_dir="output_flac"
mkdir $output_dir
regex=".*\.(xm|s3m|mod)"
raw_files=$(find . -regextype posix-extended -iregex $regex)
for i in $raw_files; do 
  filename=$(basename $i)
  file=${filename%.*}
  ffmpeg -i $i -c:a flac $output_dir/$file.flac
done

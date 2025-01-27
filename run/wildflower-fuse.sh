#!/bin/sh

img="disk.img"
path="/tmp/wildflower"

# pip install fusepy
mkdir -p $path
echo "Mounting $img in $path"
python run/wildflower-fuse.py $img $path

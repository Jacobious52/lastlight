#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
for score in stone-and-breath stone-and-breath-refuge; do
  ffmpeg -hide_banner -loglevel error -y -i "art/audio-source/$score.wav" \
    -af 'loudnorm=I=-24:TP=-3:LRA=13,afade=t=in:d=2,areverse,afade=t=in:d=10,areverse' \
    -ar 44100 -c:a libvorbis -q:a 3 "assets/audio/$score.ogg"
done

#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --locked --target wasm32-unknown-unknown --features playtest
mkdir -p dist/playtest
wasm-bindgen --target web --out-dir dist/playtest --out-name lastlight target/wasm32-unknown-unknown/debug/lastlight.wasm
cp web/index.html dist/playtest/index.html
rm -rf dist/playtest/assets
cp -R assets dist/playtest/
python3 scripts/stamp-web.py dist/playtest

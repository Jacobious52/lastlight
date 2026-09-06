#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
PROFILE="${1:-release}"
if [ "$PROFILE" = dev ]; then
  cargo build --locked --target wasm32-unknown-unknown
  WASM=target/wasm32-unknown-unknown/debug/lastlight.wasm
else
  cargo build --locked --release --target wasm32-unknown-unknown
  WASM=target/wasm32-unknown-unknown/release/lastlight.wasm
fi
mkdir -p dist
wasm-bindgen --target web --out-dir dist --out-name lastlight "$WASM"
cp web/index.html dist/index.html
touch dist/.nojekyll
rm -rf dist/assets
cp -R assets dist/

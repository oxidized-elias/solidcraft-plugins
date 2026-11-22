#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
cargo build --release --locked --target wasm32-unknown-unknown
mkdir -p dist/heatset-insert
wasm-tools component new target/wasm32-unknown-unknown/release/heatset_insert.wasm -o dist/heatset-insert/heatset-insert.wasm
cp heatset-insert/plugin.json dist/heatset-insert/
cp -r heatset-insert/icons dist/heatset-insert/
(cd dist/heatset-insert && zip -q -r -FS ../heatset-insert.zip plugin.json heatset-insert.wasm icons)
echo "Built $(pwd)/dist/heatset-insert.zip"

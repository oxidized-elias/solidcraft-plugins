#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
cargo test --locked
cargo build --release --locked --target wasm32-unknown-unknown
mkdir -p dist/breakaway-supports
wasm-tools component new target/wasm32-unknown-unknown/release/breakaway_supports.wasm -o dist/breakaway-supports/breakaway-supports.wasm
cp plugin.json README.md dist/breakaway-supports/
cp -r LICENSES icons dist/breakaway-supports/
cli=$(command -v solidcraft-cli || true)
if [[ -z $cli ]]; then cli="$HOME/.local/share/flatpak/app/io.tangencylabs.solidcraft/current/active/files/bin/solidcraft-cli"; fi
"$cli" plugin check dist/breakaway-supports
(cd dist/breakaway-supports && zip -q -r -FS ../breakaway-supports.zip plugin.json breakaway-supports.wasm README.md LICENSES icons)
python3 scripts/test_supports.py "$cli"
echo "Built $(pwd)/dist/breakaway-supports.zip"

#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
cargo test --locked
cargo build --release --locked --target wasm32-unknown-unknown
mkdir -p dist/linear-rails
wasm-tools component new target/wasm32-unknown-unknown/release/linear_rails.wasm -o dist/linear-rails/linear-rails.wasm
cp plugin.json dist/linear-rails/
cp -r icons dist/linear-rails/
cp -r tests dist/linear-rails/
cli=$(command -v solidcraft-cli || true)
if [[ -z $cli ]]; then cli="$HOME/.local/share/flatpak/app/io.tangencylabs.solidcraft/current/active/files/bin/solidcraft-cli"; fi
"$cli" plugin check dist/linear-rails
"$cli" plugin test dist/linear-rails --require-cases --out dist/test-results
python3 scripts/check_structure.py "$cli"
(cd dist/linear-rails && zip -q -r -FS ../linear-rails.zip plugin.json linear-rails.wasm icons)
cp README.md dist/linear-rails/
mkdir -p dist/linear-rails/LICENSES
cp LICENSES/SolidCraft-MIT.txt dist/linear-rails/LICENSES/
(cd dist/linear-rails && zip -q -u ../linear-rails.zip README.md LICENSES/SolidCraft-MIT.txt)
python3 scripts/test_surface.py "$cli"
echo "Built $(pwd)/dist/linear-rails.zip"

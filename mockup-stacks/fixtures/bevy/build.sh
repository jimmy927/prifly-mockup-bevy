#!/usr/bin/env bash
# needs: cargo
# Default (offline, seconds): validates the manifest and the sources are there.
# FULL=1 (network for the first crate download, about 3 to 12 minutes cold): the whole pipeline
# cargo -> wasm-bindgen -> pack-wasm -> dist/star-catcher.html. Needs wasm-bindgen (matching version) and python3.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
cd "$here"
cargo metadata --offline --no-deps --format-version 1 >/dev/null 2>&1 || cargo metadata --no-deps --format-version 1 >/dev/null
test -f src/main.rs && test -f boot.js
if [ "${FULL:-0}" != "1" ]; then echo "manifest ok; FULL=1 builds the page"; exit 0; fi

WBG="${WASM_BINDGEN:-wasm-bindgen}"
KIT="${KIT:-$(mockup-kit path)}"
TARGET="${CARGO_TARGET_DIR:-$here/target}"
export CARGO_TARGET_DIR="$TARGET"
nice cargo build --release --target wasm32-unknown-unknown -j "${JOBS:-8}"
rm -rf dist/pkg && mkdir -p dist/pkg
"$WBG" --target no-modules --no-typescript --remove-name-section --remove-producers-section --out-dir dist/pkg "$TARGET/wasm32-unknown-unknown/release/star-catcher.wasm"
WASM_OPT="${WASM_OPT:-wasm-opt}"
if command -v "$WASM_OPT" >/dev/null; then "$WASM_OPT" -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-simd -o dist/pkg/star-catcher_bg.wasm dist/pkg/star-catcher_bg.wasm; fi
python3 "$KIT/scripts/pack-wasm.py" generic \
  --wasm dist/pkg/star-catcher_bg.wasm --script dist/pkg/star-catcher.js --boot boot.js \
  --no-reload-on-hash --background '#14161c' --title "Star Catcher (Bevy)" --out dist/star-catcher.html
python3 "$KIT/scripts/check-html.py" dist/star-catcher.html
ls -l dist/star-catcher.html

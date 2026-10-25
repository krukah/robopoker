#!/usr/bin/env bash
# Builds petri to WebAssembly, binds it with the wasm-bindgen CLI that matches
# Cargo.lock (installed once under target/tools), and serves www/.
#
#   crates/petri/build.sh           build, then serve on :8787 (PORT=… to change)
#   crates/petri/build.sh build     build only
#
set -euo pipefail
cd "$(dirname "$0")/../.."
port="${PORT:-8787}"
www="crates/petri/www"
version="$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/"/, "", $3); print $3; exit }' Cargo.lock)"
bindgen="target/tools/bin/wasm-bindgen"
if [[ "$("$bindgen" --version 2>/dev/null)" != "wasm-bindgen $version" ]]; then
    cargo install --quiet --locked wasm-bindgen-cli --version "$version" --root target/tools
fi
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS='--cfg getrandom_backend="wasm_js"' \
    cargo build -p petri --release --target wasm32-unknown-unknown
"$bindgen" --target web --no-typescript --out-dir "$www/pkg" target/wasm32-unknown-unknown/release/petri.wasm
[[ "${1:-serve}" == "build" ]] && exit 0
echo "petri: http://localhost:$port"
exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$www"

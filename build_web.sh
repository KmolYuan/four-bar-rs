#!/bin/bash
set -eu

CRATE=four-bar-ui # crate name
CRATE_SNAKE_CASE="${CRATE//-/_}" # for those who name crates with-kebab-case

# This is required to enable the web_sys clipboard API which egui_web uses
# https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Clipboard.html
# https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html
export RUSTFLAGS=--cfg=web_sys_unstable_apis

# Clear output from old stuff:
rm -f docs/${CRATE_SNAKE_CASE}_bg.wasm

echo "Building rust..."
BUILD=release
cargo build --release -p ${CRATE} --lib --target wasm32-unknown-unknown

echo "Generating JS bindings for wasm..."
TARGET_NAME="${CRATE_SNAKE_CASE}.wasm"
wasm-bindgen "target/wasm32-unknown-unknown/${BUILD}/${TARGET_NAME}" \
  --out-dir docs --no-modules --no-typescript

# to get wasm-opt:  apt/brew/dnf install binaryen
if [[ "${1-}" = "-O" ]]; then
  echo "Optimizing wasm..."
  wasm-opt docs/${CRATE_SNAKE_CASE}_bg.wasm -O2 -o docs/${CRATE_SNAKE_CASE}_bg.wasm
fi

echo "Finished: docs/${CRATE_SNAKE_CASE}.wasm"

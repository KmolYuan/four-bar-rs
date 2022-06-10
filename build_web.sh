#!/usr/bin/env bash
set -eu

cd "$(dirname "${0}")" || exit
REPO=${PWD}

# This is required to enable the web_sys clipboard API which egui_web uses
# https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Clipboard.html
# https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html
export RUSTFLAGS=--cfg=web_sys_unstable_apis

echo "Generating JS bindings for wasm..."
wasm-pack build --release --out-dir ../docs/pkg -t web --no-typescript "${REPO}/four-bar-ui"
rm "${REPO}/docs/pkg/.gitignore"
rm "${REPO}/docs/pkg/package.json"
cp "${REPO}/four-bar-ui/assets/favicon.png" "${REPO}/docs"
cp "${REPO}/LICENSE" "${REPO}/docs"

echo "Make the archive..."
cd "${REPO}/docs" || exit
for RELEASE in "${REPO}/target/debug" "${REPO}/target/release"; do
  if command -v zip &> /dev/null && [ -d ${RELEASE} ]; then
    zip -r "${RELEASE}/four-bar-wasm-unknown.zip" *
  fi
done

echo "Finished"

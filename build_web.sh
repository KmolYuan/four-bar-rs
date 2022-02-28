#!/usr/bin/env bash
set -eu

cd "$(dirname "${0}")" || exit
REPODIR=${PWD}

CRATE=four-bar-ui # crate name
CRATE_SNAKE_CASE="${CRATE//-/_}" # for those who name crates with-kebab-case

# This is required to enable the web_sys clipboard API which egui_web uses
# https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Clipboard.html
# https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html
export RUSTFLAGS=--cfg=web_sys_unstable_apis

# Clear output from old stuff:
rm -f docs/${CRATE_SNAKE_CASE}_bg.wasm

echo "Generating JS bindings for wasm..."
TARGET_NAME="${CRATE_SNAKE_CASE}.wasm"
wasm-pack build --release --out-dir ../docs/pkg -t web --no-typescript "${REPODIR}/${CRATE}"
rm "${REPODIR}/docs/pkg/.gitignore"
rm "${REPODIR}/docs/pkg/package.json"
cp "${REPODIR}/four-bar-ui/src/assets/favicon.png" "${REPODIR}/docs"
cp "${REPODIR}/LICENSE" "${REPODIR}/docs"

echo "Make the archive..."
cd "${REPODIR}/docs" || exit
for RELEASE in "${REPODIR}/target/debug" "${REPODIR}/target/release"; do
  if command -v zip &> /dev/null && [ -d ${RELEASE} ]; then
    zip -r "${RELEASE}/four-bar-wasm-unknown.zip" *
  fi
done

echo "Finished"

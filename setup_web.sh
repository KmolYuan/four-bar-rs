#!/usr/bin/env bash
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/rustwasm/wasm-pack

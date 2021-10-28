#!/bin/bash
set -eu

# Pre-requisites:
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/rustwasm/wasm-pack

# For local tests with `./start_server`:
cargo install basic-http-server

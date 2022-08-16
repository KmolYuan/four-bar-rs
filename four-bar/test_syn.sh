#!/usr/bin/env bash
set -eu
cd "$(dirname "${0}")" || exit
cargo test --release --lib tests::test_syn --all-features -- --nocapture

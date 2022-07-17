#!/usr/bin/env bash
set -eu

cd "$(dirname "${0}")" || exit
REPO=${PWD}

cargo install basic-http-server
basic-http-server --addr 127.0.0.1:8080 "${REPO}/docs"

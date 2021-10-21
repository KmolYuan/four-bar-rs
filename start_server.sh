#!/bin/bash
set -eu

cd "$(dirname "${0}")" || exit
REPODIR=${PWD}

# Starts a local web-server that serves the contents of the `doc/` folder,
# which is the folder to where the web version is compiled.
basic-http-server --addr 127.0.0.1:8080 "${REPODIR}/docs"

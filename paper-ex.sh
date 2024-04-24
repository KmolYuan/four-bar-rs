#!/bin/bash
cd "$(dirname "$0")"

# Test if the "four-bar" command is available
if command -v "four-bar" &> /dev/null; then
    FB="four-bar"
else
    if ! command -v "./target/release/four-bar" &> /dev/null; then
        cargo build --release
    fi
    FB="./target/release/four-bar"
fi

# Run the paper examples
# User can pass additional flags to the command
FLAGS="--seed=0 $@"
$FB syn $FLAGS --atlas=closed.npz test-fb/mcgarva.closed.csv
$FB syn $FLAGS test-fb/yu2.closed.csv
$FB syn $FLAGS --gen=180 test-fb/bow.open.ron
$FB syn $FLAGS test-fb/wu3.partial.csv
$FB syn $FLAGS test-fb/straight-line.partial.csv

$FB syn $FLAGS --atlas=closed-sphere.npz test-sfb/mullineux64.closed.csv
$FB syn $FLAGS test-sfb/sunpos-taiwan.closed.csv
$FB syn $FLAGS test-sfb/flap.closed.csv
$FB syn $FLAGS --gen=140 test-sfb/fish.open.csv
$FB syn $FLAGS --on-unit test-sfb/circle.partial.csv

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
$FB syn $FLAGS test-fb/mcgarva.closed.csv --atlas=closed.npz
$FB syn $FLAGS test-fb/yu2.closed.csv
$FB syn $FLAGS test-fb/bow.open.ron --gen=180
$FB syn $FLAGS test-fb/wu3.partial.csv
$FB syn $FLAGS test-fb/straight-line.partial.csv

$FB syn $FLAGS test-sfb/mullineux64.closed.csv --atlas=closed-sphere.npz
$FB syn $FLAGS test-sfb/sunpos-taiwan.closed.csv
$FB syn $FLAGS test-sfb/flap.closed.csv
$FB syn $FLAGS test-sfb/fish.open.csv --gen=140
$FB syn $FLAGS test-sfb/circle.partial.csv --on-unit

$FB syn $FLAGS test-fb/yu2.closed.csv --dd --no-ref
$FB syn $FLAGS test-fb/crunode.closed.ron --dd

$FB syn $FLAGS test-mfb/hsieh1.open.csv --gen=200 --pop=2000
$FB syn $FLAGS test-mfb/hsieh2.open.ron --gen=200 --pop=2600 --res=60

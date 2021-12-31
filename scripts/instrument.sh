#!/usr/bin/env bash
set -eu

./scripts/build-wasm.sh
./scripts/build-dylib.sh

cargo profile instruments --release -t time --example perf $@
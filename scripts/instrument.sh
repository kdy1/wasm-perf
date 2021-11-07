#!/usr/bin/env bash
set -eu

./scripts/build-wasm.sh
./scripts/build-dylib.sh

cargo instruments --release -t time --bench instrument
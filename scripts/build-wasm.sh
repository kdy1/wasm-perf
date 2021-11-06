#!/usr/bin/env bash
set -eu

(cd plugin-wasm && cargo wasi build --release)

ls -al target/wasm32-wasi/release/*.wasm
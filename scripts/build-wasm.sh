#!/usr/bin/env bash
set -eu

(cd plugin-wasm && wasm-pack build --release)

ls -al plugin-wasm/pkg/*.wasm
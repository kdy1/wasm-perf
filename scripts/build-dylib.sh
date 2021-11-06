#!/usr/bin/env bash
set -eu

cargo build --release -p plugin-dylib

ls -al target/release/*.dylib
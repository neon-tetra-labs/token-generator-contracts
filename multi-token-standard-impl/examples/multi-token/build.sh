#!/bin/bash
set -e
cd "`dirname $0`"
RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release
cargo build --all --target wasm32-unknown-unknown --release
mkdir res || true
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
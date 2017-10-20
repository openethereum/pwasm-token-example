#!/bin/sh

cargo build --release --target wasm32-unknown-emscripten
wasm-build ./target token

cp ./target/token.wasm ./compiled

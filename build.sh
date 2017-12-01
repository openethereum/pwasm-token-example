#!/bin/bash

cargo build --release --target wasm32-unknown-emscripten
wasm-build ./target token

cp ./target/*.wasm ./compiled
cp ./target/json/* ./compiled

#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
wasm-build --target wasm32-unknown-unknown ./target token

cp ./target/*.wasm ./compiled
cp ./target/json/* ./compiled

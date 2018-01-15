#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
wasm-build ./target pwasm-token-contract-bin --target=wasm32-unknown-unknown --final=token --save-raw=./target/token-raw.wasm

cp ./target/*.wasm ./compiled
cp ./target/json/* ./compiled
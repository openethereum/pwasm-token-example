#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
wasm-build ./target pwasm_token_contract_bin --target=wasm32-unknown-unknown --final=token --save-raw=./target/token-deployed.wasm

cp ./target/*.wasm ./compiled
cp ./target/json/* ./compiled

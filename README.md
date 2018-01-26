[![Build Status](https://travis-ci.org/paritytech/pwasm-token-example.svg?branch=master)](https://travis-ci.org/paritytech/pwasm-token-example)
## Build prerequisites
- rust with `wasm32-unknown-unknown` target
```
rustup target add wasm32-unknown-unknown --toolchain nightly
```
- wasm build util, run
```
cargo install --git https://github.com/paritytech/wasm-utils wasm-build
```
## Build
```
./build.sh
```
## Testing
```
cargo test --manifest-path="contract/Cargo.toml" --features std
```

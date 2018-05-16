[![Build Status](https://travis-ci.org/paritytech/pwasm-token-example.svg?branch=master)](https://travis-ci.org/paritytech/pwasm-token-example)
## Build prerequisites
Install rust with `wasm32-unknown-unknown` target:
```
rustup target add wasm32-unknown-unknown
```
Install Wasm build util:
```
cargo install pwasm-utils-cli
```
## Build
Run:
```
./build.sh
```
## Testing
```
cargo test --manifest-path="contract/Cargo.toml" --features std
```

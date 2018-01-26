[![Build Status](https://travis-ci.org/paritytech/pwasm-token-example.svg?branch=master)](https://travis-ci.org/paritytech/pwasm-token-example)
## Build prerequisites
- rust with `wasm32-unknown-unknown` target - instruction to setup can be found [here](https://www.hellorust.com/news/native-wasm-target.html)
- wasm build util, run `cargo install --git https://github.com/paritytech/wasm-utils wasm-build` to install
- bash to run `./build.sh`
## Build
`./build.sh`
## Testing
`cargo test --manifest-path="contract/Cargo.toml" --features std`

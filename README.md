## Build prerequisites
- rust with `wasm32-unknown-emscripten` target - instruction to setup can be found [here](https://hackernoon.com/compiling-rust-to-webassembly-guide-411066a69fde)
- make sure `emcc` tool is in the `PATH` since build script uses it internally
- wasm build util, run `cargo install --git https://github.com/paritytech/wasm-utils wasm-build` to install
- bash to run `./build.sh`
## Build
- `./build.sh`
## Testing
`cargo test --features std`

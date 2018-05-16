#!/bin/bash

# this script is intended to be used from .travis.yml

curl -sL https://storage.googleapis.com/wasm-llvm/builds/linux/$WATERFALL_BUILD/wasm-binaries.tbz2 | tar xvkj

#!/bin/sh

cargo fmt
cargo build --target=wasm32-unknown-unknown &&
wasm-gc target/wasm32-unknown-unknown/debug/tetris.wasm html/test.wasm


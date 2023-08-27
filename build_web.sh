#!/bin/bash

cargo build --target wasm32-unknown-unknown &&
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/debug/chess2.wasm &&
rm -rf out/assets &&
cp -r assets out/
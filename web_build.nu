#!/usr/bin/env nu

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/rusty_rogue.wasm --out-dir wasm --no-modules --no-typescript

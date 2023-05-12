# Rusty Rogue

## Play online

A web-browser version of the game is available [here](https://bbarker.github.io/RustyRogue/wasm/).

## Building for the Browser

A full guide with which most of these notes are based on are located
at [in the Roguelike Tutorial - In Rust](https://bfnightly.bracketproductions.com/webbuild.html).


### Initial Setup

- `rustup target add wasm32-unknown-unknown`
- `cargo install wasm-bindgen-cli`
- `cargo install cargo-server` for testing locally

### Building and Running Locally
- `./web_build.nu`
- `cargo server --path ./wasm`

### Recurring gotchas

-  `wasm-bindgen` needs to be reinstalled when the Rust toolchain is updated.
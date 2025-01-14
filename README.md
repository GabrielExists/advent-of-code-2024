# Advent of code 2024

My playthrough of [Advent of code](https://adventofcode.com/) for 2024.
I'm using Rust, and yew to just display the results

## Installation
Install rustup from https://rustup.rs, then run
```
rustup target add wasm32-unknown unknown
cargo install binstall
cargo binstall --locked trunk
```
OR
```
rustup target add wasm32-unknown unknown
# Make sure you have openssl on your system
cargo install --locked trunk
```

## Running
```
trunk serve
```
Then navigate to the port shown in the output, for example http://127.0.0.1:8080/
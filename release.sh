#!/bin/bash

here=$(pwd)

cargo build --target=wasm32-unknown-emscripten 
cp src/index.html target/wasm32-unknown-emscripten/debug
cd target/wasm32-unknown-emscripten/debug
zip wasm32-unknown-emscripten.zip bus_zombie_rust.wasm.map bus_zombie_rust.wasm bus-zombie-rust.js \
bus-zombie-rust.d warning.wav scoop.wav hit.wav index.html hit.wav
mv wasm32-unknown-emscripten.zip $here

cd $here
cargo build
rsync -avz resources target/debug
cd target/debug
zip -r bus-zombie-rust-linux.zip bus-zombie-rust resources
mv bus-zombie-rust-linux.zip $here


cd $here
export PKG_CONFIG_ALLOW_CROSS=1
cargo build --target x86_64-pc-windows-gnu --features soundoff
rsync -avz windows/SDL2.dll resources target/x86_64-pc-windows-gnu/debug
cd target/x86_64-pc-windows-gnu/debug
zip -r bus-zombie-rust-windows.zip bus-zombie-rust.exe SDL2.dll resources
mv bus-zombie-rust-windows.zip $here


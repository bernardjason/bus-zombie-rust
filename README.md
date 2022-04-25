# Exploding Zombies vs Buses

A 3d rust game using OpenGl and Emscripten to build for the wasm32-unknown-emscripten.
It can also run standalone, developed and tested on Linux but will
work on Windows, see some of the other Rust projects in this repo.

![screenshot](screenshot.png)


I hosted the WASM version here TBD...

to run standalone
```
cargo build
cargo run
```

For web deployment. You may need to do
```
rustup target add wasm32-unknown-emscripten
```

then this will work
```
cargo build --target=wasm32-unknown-emscripten 
```

To try web version locally having built to emscripten target try script
```
./run_wasm.sh
```

on linux to target windows without sound as there is a high pitch noise I need to look at
```
export PKG_CONFIG_ALLOW_CROSS=1
cargo build --target x86_64-pc-windows-gnu --features soundoff
rsync -avz windows/SDL2.dll resources target/x86_64-pc-windows-gnu/debug
cd target/x86_64-pc-windows-gnu/debug
zip -r bus-zombie-rust.zip bus-zombie-rust.exe SDL2.dll resources

```

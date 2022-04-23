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

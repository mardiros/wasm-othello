# Othello Game on WebAssembly written in Rust

## Development status

WIP - you cannot play right now.


## Development setup

Necessary compiler target `wasm32-unknown-unknown` (Rust nightly only)

```
$ rustup target add wasm32-unknown-unknown
$ cargo +nightly install cargo-web
```

Start a web dev server

```
$ cargo +nightly web start
```

## Build

```
$ cargo +nightly web build
```

## Reference
* https://github.com/koute/cargo-web
* https://github.com/koute/stdweb
* https://github.com/DenisKolodin/yew
* https://developer.mozilla.org/en-US/docs/WebAssembly

## License

MIT

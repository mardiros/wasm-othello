export RUST_BACKTRACE = "1"
export RUST_LOG="othello-server"

dev: debug-client start-server

start-server:
    cd othello-server && cargo run

debug-client:
    cargo +nightly web build --target wasm32-unknown-unknown -p othello-client
    cp target/wasm32-unknown-unknown/release/othello-client.js static
    cp target/wasm32-unknown-unknown/release/othello-client.wasm static

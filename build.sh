# build
#cargo build --target wasm32-unknown-unknown --release

# build wasm to www
# file name matches output file (cargo package name) above
#wasm-bindgen ./target/wasm32-unknown-unknown/release/rust_web_roguelike.wasm --out-dir docs --web --no-typescript

wasm-pack build --out-name rust_web_roguelike --out-dir docs --target web --no-typescript
cargo build --release --target wasm32-unknown-unknown
wasm-opt target/wasm32-unknown-unknown/release/generals.wasm --generate-global-effects -Oz -tnh --flatten --rereloop --converge -Oz -o docs/generals.wasm

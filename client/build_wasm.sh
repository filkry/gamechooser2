#!/bin/sh
WASMFILE=rs/target/wasm32-unknown-unknown/debug/gamechooser2_client.wasm
OUTDIR=served_files/rs-wasm-bindgen-output

cargo build --manifest-path=rs/Cargo.toml --target wasm32-unknown-unknown
mkdir $OUTDIR
wasm-bindgen --target web --no-typescript --out-dir $OUTDIR $WASMFILE
copy $WASMFILE $OUTDIR
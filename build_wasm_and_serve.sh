#!/bin/sh

cd client
./build_wasm.sh
cd ..
cd server
cargo run
cd ..

#!/bin/sh

cd client
source build_wasm.sh
cd ..
cd server
cargo run
cd ..

set wasmfile=rs\target\wasm32-unknown-unknown\debug\gamechooser2_client.wasm
set outdir=served_files\rs-wasm-bindgen-output

cargo build --manifest-path=rs/Cargo.toml --target wasm32-unknown-unknown
mkdir %outdir%
wasm-bindgen\wasm-bindgen.exe --target web --no-typescript --out-dir %outdir% %wasmfile%
copy %wasmfile% %outdir%
set wasmfile=rs\target\wasm32-unknown-unknown\debug\gamechooser2_client.wasm
set outdir=rs-wasm-bindgen-output

mkdir outdir
wasm-bindgen\wasm-bindgen.exe --target web --no-typescript --out-dir %outdir% %wasmfile%
copy %wasmfile% %outdir%
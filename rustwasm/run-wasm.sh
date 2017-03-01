#!/bin/sh -eux

rustc -V
emcc -v

cargo build --target wasm32-unknown-emscripten

rm -rf dist/
mkdir dist/
cp static/* dist/
cp target/wasm32-unknown-emscripten/debug/deps/step4*.wasm dist/lib.wasm
# exclude .asm.js
cp target/wasm32-unknown-emscripten/debug/deps/step4*[0-9a-f].js dist/lib.js

echo "open http://localhost:8080/"
cd dist/ && python -m SimpleHTTPServer 8080

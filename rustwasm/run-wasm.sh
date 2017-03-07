#!/bin/sh -eux

rustc -V
emcc -v

cargo build --target wasm32-unknown-emscripten

rm -rf dist/ && mkdir dist/
cp static/* dist/
# emscripten's bootstrap
cp target/wasm32-unknown-emscripten/debug/deps/step4*[0-9a-f].js dist/lib.js
# wasm
cp target/wasm32-unknown-emscripten/debug/deps/step4*.wasm dist/lib.wasm

echo "open http://localhost:8080/"
cd dist/ && python -m SimpleHTTPServer 8080

#!/bin/sh -eux

TARGET=step5

rustc -V
emcc -v

cargo build --target wasm32-unknown-emscripten

rm -rf dist/ && mkdir dist/
cp static/* dist/
# emscripten's bootstrap
cp target/wasm32-unknown-emscripten/debug/deps/${TARGET}*[0-9a-f].js dist/lib.js
# wasm
cp target/wasm32-unknown-emscripten/debug/deps/${TARGET}*.wasm dist/lib.wasm

echo "open http://localhost:8080/"
cd dist/ && python -m SimpleHTTPServer 8080

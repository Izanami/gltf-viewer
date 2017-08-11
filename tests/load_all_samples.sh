#!/bin/bash
# NOTE: should be called from crate root!

set -e
mode=${1:-release} # pass `debug` instead to test a debug build

mkdir -p target/screenshots
export CARGO_INCREMENTAL=1
if [[ "$mode" == "release" ]]; then
    cargo build --release
else
    cargo build
fi
for file in ../gltf/glTF-Sample-Models/2.0/**/glTF/*.gltf; do
    model_name=$(basename "$file" .gltf)
    target/"$mode"/gltf-viewer "$file" -s target/screenshots/"$model_name".png
done

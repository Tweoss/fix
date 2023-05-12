#!/usr/bin/bash
cargo build --target=wasm32-wasi --release
cd ../fix-ref; cargo run ../testing/target/wasm32-wasi/release/helloworld.wasm -o ../testing/output.wasm; cd ../testing
~/fix/build/applications-prefix/src/applications-build/_deps/wasm-tools-build/src/module-combiner/wasmlink --enable-multi-memory \
           --enable-exceptions \
           output.wasm \
           /home/fchua/fix/applications/util/wasi_snapshot_preview1.wasm \
           -m wasi_command \
           -n wasi_snapshot_preview1 \
           -o sys_linked.wasm
~/fix/build/applications-prefix/src/applications-build/_deps/wasm-tools-build/src/module-combiner/wasmlink \
        --enable-multi-memory \
        --enable-exceptions \
        sys_linked.wasm \
        /home/fchua/fix/applications/util/fixpoint_storage.wasm \
        -n fixpoint_storage \
        -o linked_with_storage.wasm

~/fix/build/src/tester/stateless-tester tree:4 string:unused file:./linked_with_storage.wasm uint32:7 file:./addblob.wasm
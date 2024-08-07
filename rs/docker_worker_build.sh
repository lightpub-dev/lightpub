#!/bin/bash
if [ "$1" = "debug" ]; then
    cargo build --bin lightpub_worker && cp target/debug/lightpub_worker /out/lightpub_worker
elif [ "$1" = "release" ]; then
    cargo build --release --bin lightpub_worker && cp target/release/lightpub_worker /out/lightpub_worker
else
    echo "Usage: $0 [debug|release]" && exit 1
fi

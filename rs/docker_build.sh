#!/bin/bash
if [ "$1" = "debug" ]; then
    cargo build --package lightpub_api && cp target/debug/lightpub_api /out/lightpub_api
elif [ "$1" = "release" ]; then
    cargo build --release --package lightpub_api && cp target/release/lightpub_api /out/lightpub_api
else
    echo "Usage: $0 [debug|release]" && exit 1
fi

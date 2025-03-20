#!/bin/bash

if [ $# -eq 0 ]; then
    echo "Error: Build type argument required (debug/release)"
    exit 1
fi

case "$1" in
    "debug")
        cargo build
        ;;
    "release")
        cargo build --release
        ;;
    *)
        echo "Error: Invalid build type. Use 'debug' or 'release'"
        exit 1
        ;;
esac

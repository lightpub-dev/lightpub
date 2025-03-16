#!/bin/sh
./generate-jwt-keys.sh
exec ./lightpub_rs

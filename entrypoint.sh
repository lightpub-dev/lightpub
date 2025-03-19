#!/bin/sh
./generate-jwt-keys.sh && \
./generate-vapid-keys.sh && \
exec ./lightpub_rs

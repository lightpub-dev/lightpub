#!/bin/sh
if [ ! -f data/jwt.pem ]; then
    openssl genrsa -out data/jwt.pem 4096
    openssl rsa -in data/jwt.pem -pubout -out data/jwtpub.pem
fi

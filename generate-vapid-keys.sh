#!/bin/sh
if [ ! -f data/vapid.pem ]; then
    openssl ecparam -genkey -name prime256v1 -out data/vapid.pem
fi

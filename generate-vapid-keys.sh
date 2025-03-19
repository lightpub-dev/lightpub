#!/bin/sh
if [ ! -f data/vapid.pem ]; then
    openssl ecparam -genkey -name prime256v1 -out data/vapid.pem && \
    openssl ec -in data/vapid.pem -pubout -outform DER|tail -c 65|base64|tr '/+' '_-'|tr -d '\n=' > data/vapid_pub.pem
fi

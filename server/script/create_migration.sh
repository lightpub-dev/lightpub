#!/bin/sh
if [ -z "$1" ]; then
  echo "Please provide a name for the migration"
  exit 1
fi
migrate create -ext sql -dir migrations/ -seq $1

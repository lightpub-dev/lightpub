#!/bin/bash

if [ "$WORKERS_POST" = "" ]; then
  echo "WORKERS_POST is not set. Exiting."
  exit 1
fi
if [ "$WORKERS_FETCH" = "" ]; then
  echo "WORKERS_FETCH is not set. Exiting."
  exit 1
fi

RUN_FILE=/tmp/lightpub_worker_running
rm -f $RUN_FILE && lightpub_worker --post-worker $WORKERS_POST --fetch-worker $WORKERS_FETCH --generate-run-file $RUN_FILE

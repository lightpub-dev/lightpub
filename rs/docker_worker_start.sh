#!/bin/bash

RUN_FILE=/tmp/lightpub_worker_running
rm -f $RUN_FILE && touch db/db.sqlite3 && lightpub_worker --generate-run-file $RUN_FILE

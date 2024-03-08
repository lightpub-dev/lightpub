#!/bin/bash
CONCURRENT=${1:-2}
watchmedo auto-restart -p '*.py' -d . --recursive -- celery -A lightpub worker -E -l INFO -c $CONCURRENT


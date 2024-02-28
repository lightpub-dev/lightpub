#!/bin/bash
watchmedo auto-restart -p '*.py' -d . --recursive -- celery -A lightpub worker -E -l INFO

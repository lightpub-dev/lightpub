#!/bin/bash
touch db.sqlite3 && refinery migrate && lightpub_api

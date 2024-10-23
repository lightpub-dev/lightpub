#!/bin/sh
migrate -path ./migrations -database sqlite://db/db.sqlite3 up

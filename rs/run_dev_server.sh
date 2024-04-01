#!/bin/bash
cargo sqlx prepare && \
    docker compose -f docker-compose.fed.yml --profile lightpub down && \
    docker-compose -f docker-compose.fed.yml --profile lightpub up --build

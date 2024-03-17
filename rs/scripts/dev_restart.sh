#!/bin/bash
docker compose -f docker-compose.fed.yml  --profile lightpub --profile mastodon --profile misskey down lightpub_api nginx && \
docker compose -f docker-compose.fed.yml  --profile lightpub --profile mastodon --profile misskey up lightpub_api nginx -d --build

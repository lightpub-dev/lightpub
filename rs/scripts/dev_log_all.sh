#!/bin/bash
docker compose -f docker-compose.fed.yml  --profile lightpub --profile mastodon --profile misskey logs -f

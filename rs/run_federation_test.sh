#!/bin/bash -x
echo "=== TEST ==="
docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub --profile testing down -v
docker compose -f docker-compose.fed.yml --profile misskey --profile lightpub --profile testing up --build --exit-code-from test-runner

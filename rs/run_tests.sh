#!/bin/bash -x
echo "=== TEST ==="
docker compose -f docker-compose.test.yml down
docker compose -f docker-compose.test.yml up --build --exit-code-from test-runner

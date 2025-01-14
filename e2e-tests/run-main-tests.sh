#!/usr/bin/env bash

set -euo pipefail

if [ "${ENABLE_SETUP:-}" = "true" ]; then
  node src/setup.js
fi

k6 run \
  -e "API_ORIGIN=$API_ORIGIN" \
  -e "TARGET_URL=$TARGET_URL" \
  -e "SERVICE_TOKEN=$SERVICE_TOKEN" \
  -e "ORGANIZATION_ID=$ORGANIZATION_ID" \
  -e "VUS=$VUS" \
  -e "ITERATIONS=$ITERATIONS" \
  -e "DURATION=$DURATION" \
  src/main.js

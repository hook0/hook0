#!/usr/bin/env bash

set -euo pipefail

if [ "${ENABLE_SETUP:-}" = "true" ]; then
  node src/setup.js
fi

k6 run \
  -e "VUS=$VUS" \
  -e "ITERATIONS=$ITERATIONS" \
  -e "DURATION=$DURATION" \
  -e "API_ORIGIN=$API_ORIGIN" \
  -e "TARGET_URL=$TARGET_URL" \
  -e "SERVICE_TOKEN=$SERVICE_TOKEN" \
  -e "ORGANIZATION_ID=$ORGANIZATION_ID" \
  -e "N_EVENT_TYPES=$N_EVENT_TYPES" \
  -e "N_SUBSCRIPTIONS=$N_SUBSCRIPTIONS" \
  -e "N_LABELS=$N_LABELS" \
  src/load.ts

#!/usr/bin/env bash
# Create a Hook0 event type named "<service>.<resource>.<verb>".
# Usage: create_event_type.sh <application_id> <service> <resource> <verb>
# Requires env: HOOK0_API (e.g. https://api.hook0.com/api/v1), HOOK0_TOKEN
set -euo pipefail

APP_ID="${1:?application_id required}"
SERVICE="${2:?service required}"
RESOURCE="${3:?resource required}"
VERB="${4:?verb required}"
: "${HOOK0_API:?set HOOK0_API}"
: "${HOOK0_TOKEN:?set HOOK0_TOKEN}"

curl -fsS -X POST "$HOOK0_API/event_types/" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"application_id\":\"$APP_ID\",\"service\":\"$SERVICE\",\"resource_type\":\"$RESOURCE\",\"verb\":\"$VERB\"}"
echo
echo "Created event type: $SERVICE.$RESOURCE.$VERB"

#!/usr/bin/env bash
# Send one event to Hook0. A fresh UUID is generated as the idempotency key.
# Usage: send_event.sh <application_id> <event_type> <json_payload> [labels_json]
# labels_json routes the event to subscriptions filtering on those labels, e.g. '{"tenant":"acme"}'.
# Requires env: HOOK0_API (e.g. https://api.hook0.com/api/v1), HOOK0_TOKEN
set -euo pipefail

APP_ID="${1:?application_id required}"
EVENT_TYPE="${2:?event_type required}"
PAYLOAD="${3:?json payload required}"
LABELS="${4:-}"
[ -n "$LABELS" ] || LABELS="{}"
: "${HOOK0_API:?set HOOK0_API}"
: "${HOOK0_TOKEN:?set HOOK0_TOKEN}"

# Generate a UUID for event_id (idempotency key) with the tool that is available.
if command -v uuidgen >/dev/null 2>&1; then
  EVENT_ID="$(uuidgen | tr 'A-Z' 'a-z')"
elif [ -r /proc/sys/kernel/random/uuid ]; then
  EVENT_ID="$(cat /proc/sys/kernel/random/uuid)"
else
  EVENT_ID="$(python3 -c 'import uuid;print(uuid.uuid4())')"
fi

OCCURRED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Embed the caller's JSON payload as a string (Hook0 expects payload as a string).
BODY="$(python3 - "$APP_ID" "$EVENT_ID" "$EVENT_TYPE" "$PAYLOAD" "$OCCURRED_AT" "$LABELS" <<'PY'
import json, sys
app_id, event_id, event_type, payload, occurred_at, labels = sys.argv[1:7]
print(json.dumps({
    "application_id": app_id,
    "event_id": event_id,
    "event_type": event_type,
    "payload": payload,
    "payload_content_type": "application/json",
    "occurred_at": occurred_at,
    "labels": json.loads(labels),
}))
PY
)"

curl -fsS -X POST "$HOOK0_API/event/" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$BODY"
echo
echo "Sent $EVENT_TYPE (event_id=$EVENT_ID)"

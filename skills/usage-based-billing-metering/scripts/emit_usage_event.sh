#!/usr/bin/env bash
# Emit an idempotent usage/metering event to Hook0.
# The natural idempotency key is hashed into a stable UUIDv5 for event_id, so
# re-sending the same key never double-counts.
# Usage: emit_usage_event.sh <application_id> <customer_id> <meter> <quantity> <idempotency_key>
# Requires env: HOOK0_API (e.g. https://api.hook0.com/api/v1), HOOK0_TOKEN
set -euo pipefail

APP_ID="${1:?application_id required}"
CUSTOMER_ID="${2:?customer_id required}"
METER="${3:?meter required}"
QUANTITY="${4:?quantity required}"
IDEM_KEY="${5:?idempotency_key required}"
: "${HOOK0_API:?set HOOK0_API}"
: "${HOOK0_TOKEN:?set HOOK0_TOKEN}"

OCCURRED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

BODY="$(python3 - "$APP_ID" "$CUSTOMER_ID" "$METER" "$QUANTITY" "$IDEM_KEY" "$OCCURRED_AT" <<'PY'
import json, sys, uuid
app_id, customer_id, meter, quantity, idem_key, occurred_at = sys.argv[1:7]
# Stable UUIDv5 from the natural idempotency key -> Hook0 dedups on event_id.
event_id = str(uuid.uuid5(uuid.NAMESPACE_URL, f"usage:{app_id}:{idem_key}"))
payload = json.dumps({"customer_id": customer_id, "meter": meter,
                      "quantity": float(quantity), "idempotency_key": idem_key})
print(json.dumps({
    "application_id": app_id,
    "event_id": event_id,
    "event_type": "usage.recorded",
    "payload": payload,
    "payload_content_type": "application/json",
    "occurred_at": occurred_at,
    "labels": {"customer_id": customer_id, "meter": meter},
}))
PY
)"

curl -fsS -X POST "$HOOK0_API/event/" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$BODY"
echo
echo "Recorded usage: customer=$CUSTOMER_ID meter=$METER qty=$QUANTITY (idem=$IDEM_KEY)"

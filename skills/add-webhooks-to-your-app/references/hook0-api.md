# Hook0 API — quick reference

Base URL (cloud): `https://api.hook0.com/api/v1`  (self-hosted: your own host + `/api/v1`)
Auth: `Authorization: Bearer <TOKEN>` — use a **service token** for automated
systems, or an application secret. Full docs: https://documentation.hook0.com

Set once:
```bash
export HOOK0_API="https://api.hook0.com/api/v1"
export HOOK0_TOKEN="<your service token>"
export APP_ID="<your application uuid>"
```

## Create an event type — `POST /event_types/`
```bash
curl -X POST "$HOOK0_API/event_types/" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"application_id":"'"$APP_ID"'","service":"billing","resource_type":"invoice","verb":"paid"}'
```
Produces the event type `billing.invoice.paid`. List them with
`GET /event_types/?application_id=$APP_ID`.

## Send an event — `POST /event/` (singular)
```bash
curl -X POST "$HOOK0_API/event/" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "event_id": "69f69ecf-0d9e-4c92-a6e0-3b2676343940",
    "event_type": "billing.invoice.paid",
    "payload": "{\"invoice_id\":\"inv_123\"}",
    "payload_content_type": "application/json",
    "occurred_at": "2026-01-15T10:30:00Z",
    "labels": {"tenant": "acme"}
  }'
```
Notes:
- `event_id` is optional (the server generates one) but, when you set it, it must
  be a UUID unique per occurrence. It is your idempotency key: a retry with the
  same `event_id` is refused as already ingested instead of being delivered twice.
- The event type must exist first (`POST /event_types/`), or the event is refused.
- `payload` is a string; set `payload_content_type` accordingly.
- `labels` is required (`{}` if you have none) and drives routing: a subscription with a matching label filter receives it.
- Response: `{"application_id": "...", "event_id": "...", "received_at": "..."}`.

## Create a subscription — `POST /subscriptions/`
```bash
curl -X POST "$HOOK0_API/subscriptions/" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["billing.invoice.paid"],
    "labels": {"tenant": "acme"},
    "target": {"type": "http", "method": "POST", "url": "https://customer.example.com/hooks", "headers": {}}
  }'
```
The subscription only receives events whose labels match (`tenant=acme` here), so
one application safely fans out to many tenants.

## Inspect deliveries
- `GET /events/?application_id=$APP_ID` — events received by Hook0.
- `GET /request_attempts/?application_id=$APP_ID` — each delivery try + status.
- `POST /events/{EVENT_ID}/replay` — replay a delivery (body: `{"application_id": "..."}`).

> Field names can evolve; treat this as a starting point and confirm against the
> live reference at https://documentation.hook0.com before shipping.

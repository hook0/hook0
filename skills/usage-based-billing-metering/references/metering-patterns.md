# Metering patterns

## Idempotency key design
A usage event must be countable exactly once even if sent 2+ times. Build the key
from something stable about the *source occurrence*, not the send moment:

- Good: `cus_acme:api_requests:req_9f3c1` (request id), `cus_acme:seats:2026-01`
  (period-scoped snapshot), `cus_acme:gb_egress:2026-01-15T10` (hour bucket).
- Bad: a fresh UUID per send (retries duplicate), `now()` (same increment sent
  twice yields two keys).

With Hook0, put this stable value where dedup happens: send it as the event's
`event_id` (must be a UUID — hash your natural key into a UUIDv5 if it isn't one)
so replays and at-least-once producers don't double-count.

## Aggregation cheatsheet
| Meter kind | Aggregation | Example |
| --- | --- | --- |
| Consumption (API calls, egress, events) | `sum` over period | `api_requests` |
| Peak resource (seats, concurrent) | `max` over period | `active_seats` |
| Distinct actors | `unique_count` | `monthly_active_users` |
| Gauge / latest | `last` | `stored_gb` (snapshot) |

## Threshold notifications
Fire at meaningful fractions of the entitlement, not every increment:

```
on increment:
  usage = period_sum(customer, meter)
  for pct in (0.8, 1.0):
    if crossed(pct, previous_usage, usage, quota):
      emit "usage.threshold.reached" { customer, meter, pct, usage, quota }
  if usage > quota and plan.blocks_on_overage:
    emit "usage.limit.exceeded" { customer, meter, usage, quota }
```

`crossed()` must be edge-triggered (fire once when passing the line), not level —
otherwise every subsequent increment re-fires. Persist the last notified level per
customer+meter.

## Reconciliation loop
1. Nightly, sum raw usage events per customer+meter for the open period.
2. Read what the billing provider has recorded for the same period.
3. Alert on drift beyond a tolerance; push the delta (never overwrite).
4. Keep raw events for the dispute window — they are the source of truth.

## Plan model sketch
```json
{
  "plan": "pro",
  "meters": {
    "api_requests": {"included": 100000, "overage_price_per_unit": "0.0001", "aggregation": "sum"},
    "active_seats": {"included": 5, "overage_price_per_unit": "12.00", "aggregation": "max"}
  }
}
```
Keep prices as strings/decimals, never floats.

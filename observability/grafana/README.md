# Hook0 observability — Grafana alerting as-code

Grafana alert rules for Hook0, kept as-code so they are reviewable and reproducible.

- **Grafana**: https://hook0-grafana.france-nuage.fr (folder `Hook0`, uid `ff485ab4jp6gwc`).
- **Metrics backend**: Grafana Mimir (self-hosted, France Nuage) — datasource uid `mimir`. Hook0 pushes via OTLP.
- **Prod DB**: Clever Cloud Postgres — datasource uid `bev6em79t7jlsc`.
- **Contact point**: `Better Uptime` (existing).

## Why these rules exist

On 2 Aug 2026 a production incident saw a large inbound-event spike for one customer (~134k events in a day,
~1000 duplicate webhooks) with ~40 min worker lag and a ~2k delivery backlog. **No alert fired**: the only
rule (`Unfinished request attempts`) only watches attempts on *disabled/deleted* subscriptions, and it was
silent on NoData. These rules close that gap.

## Rules

| File | Datasource | Fires when | Notes |
|------|-----------|-----------|-------|
| `inbound-events-global-spike.json` | Mimir | `sum(rate(events_ingested_total[10m])) > 50` for 10m | Normal P99 ≈ 16 ev/s; incident peaked ~3500 ev/s. Per-tenant drill-down is manual (view `event.events_per_day`) to avoid metric cardinality. |
| `delivery-backlog-pastdue.json` | Postgres | past-due pending attempts (`delay_until <= now()-10min`) `> 100` for 10m | Filters on `delay_until` (real lag), NOT `created_at` (would count ~78k healthy scheduled retries). `noData`/`execErr` = Alerting. |
| `unfinished-request-attempts.json` | Postgres | orphaned attempts on disabled/deleted subs `> 1000` | Pre-existing rule; **change vs prod: `noDataState`/`execErrState` NoData/Error → Alerting** so a broken query pages instead of going silent. |

## Apply (requires the Grafana service-account token)

```sh
# token in ~/.claude/shared/secrets/grafana-hook0.env  (key: access_token)
BASE=https://hook0-grafana.france-nuage.fr
TOKEN=...   # access_token

# create the two new rules (X-Disable-Provenance keeps them editable in the UI)
for f in inbound-events-global-spike delivery-backlog-pastdue; do
  curl -sS -X POST "$BASE/api/v1/provisioning/alert-rules" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "X-Disable-Provenance: true" \
    --data-binary @observability/grafana/alerts/$f.json
done

# update the existing rule (NoData/Error -> Alerting)
curl -sS -X PUT "$BASE/api/v1/provisioning/alert-rules/cf485b748dts0d" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -H "X-Disable-Provenance: true" \
  --data-binary @observability/grafana/alerts/unfinished-request-attempts.json
```

## Not yet covered (needs code)

- **Worker lag (seconds) & delivery-outcome ratio**: the Pulsar consumer gauges are not emitted in prod
  (`pulsar_consumer_stats_interval` disabled + flagged unstable). Worker-side Mimir metrics (M1 lag at
  attempt pickup, M2 `outcome` counter) must be added in `output-worker` before Mimir-based lag/outcome alerts.

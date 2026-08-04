# Hook0 observability — Grafana alerting as-code

Grafana alert rules for Hook0, kept as-code so they are reviewable and reproducible.

- **Grafana**: https://hook0-grafana.france-nuage.fr (folder `Hook0`, uid `ff485ab4jp6gwc`).
- **Metrics backend**: Grafana Mimir (self-hosted, France Nuage) — datasource uid `mimir`. Hook0 pushes via OTLP.
- **Prod DB**: Clever Cloud Postgres — datasource uid `bev6em79t7jlsc`.
- **Contact point**: `Better Uptime` (existing).

## Why these rules exist

A production inbound-event spike went undetected. The only alert rule that existed
(`Unfinished request attempts`) watches attempts on *disabled/deleted* subscriptions only, and it was
silent on NoData — so a live ingestion spike, a delivery backlog, and a broken query all passed unnoticed.
These rules close that gap: spike detection, past-due backlog, and making a failing query page instead of
going quiet.

## Rules

| File | Datasource | Fires when | Notes |
|------|-----------|-----------|-------|
| `inbound-events-global-spike.json` | Mimir | `sum(rate(events_ingested_total[10m])) > 50` for 10m | 50 ev/s sits well above normal steady-state, so it catches a real spike with headroom. Per-tenant drill-down is manual (view `event.events_per_day`) to avoid metric cardinality. |
| `delivery-backlog-pastdue.json` | Postgres | past-due pending attempts (`delay_until <= now()-10min`) `> 100` for 10m | Filters on `delay_until` (real lag), NOT `created_at` (which would count the healthy pool of scheduled retries). `noData`/`execErr` = Alerting. |
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

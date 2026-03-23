# Frontend Redesign - Deferred Features

## Real-time & Charts (deferred from v1 redesign)

### Real-time Event Stream
- WebSocket/SSE connection for live event feed on app dashboard
- Live event feed widget showing events as they arrive
- Connection status indicator (connected/reconnected/offline)
- Auto-reconnect with exponential backoff

### Dashboard Charts
- App dashboard: delivery success/failure rate chart (last 24h)
- App dashboard: latency histogram per subscription
- App dashboard: events volume over time (line chart)
- Org dashboard: aggregated metrics across all apps
- Chart library already pinned: Chart.js + vue-chartjs

### Real-time Delivery Status
- Active subscriptions health indicator (last delivery success/failure)
- Polling-based near-real-time as intermediate step (TanStack Query polling every 5-30s)
- Consider WebSocket for true real-time when backend supports it

## Event Composer / Playground (deferred from v1 redesign)
- Dedicated page/modal for composing JSON event payloads
- Event type selector with schema-based autocomplete
- Payload validation against event type JSON schema
- Send test event with inline response preview
- History of recent test events

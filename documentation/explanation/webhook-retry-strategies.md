---
title: "Why Most Webhook Retry Strategies Fail — and How to Fix Yours"
description: "Fixed interval, exponential backoff, jitter, and two-phase retry compared side by side, with real failure scenarios and how Hook0's own retry schedule works."
keywords: [webhook retry strategy, exponential backoff webhook, webhook retry best practices, webhook delivery retry, retry with jitter, two-phase retry]
---

# Webhook retry strategies compared

3-5% of webhook deliveries fail on the first attempt. Network blips, rolling deploys, load balancer drains, brief DNS hiccups. These are transient failures — the endpoint is fine a few seconds later.

Without retries, those events are gone. With naive retries, you risk making things worse. Your strategy determines whether you recover quietly or trigger a cascading outage.

## Strategy comparison

| Strategy | Recovery rate | Thundering herd risk | Complexity | When to use |
|----------|:---:|:---:|:---:|-------------|
| Fixed interval | ~85% | **High** | Low | Dev/test environments only |
| Exponential backoff | ~92% | Medium | Low | Single-consumer systems |
| Exponential + jitter | ~95% | **Low** | Medium | Multi-consumer systems |
| Two-phase (fast then slow) | Very high | **Low** | Medium | Production webhook infrastructure |
| Circuit breaker | Varies | **None** | High | When you control both sides |

## Fixed interval: why it breaks at scale

Say you have a 10-second fixed retry and 1,000 subscribers hitting a recovering endpoint:

- **t=0s**: 1,000 deliveries fail (endpoint down)
- **t=10s**: 1,000 retries hit at the same time
- **t=20s**: 1,000 retries again -- the endpoint just came back but gets 1,000 concurrent requests

The endpoint either recovers and immediately gets hammered, or stays down because the retry storm prevents recovery. A transient failure becomes a sustained outage.

## Exponential backoff: better but not enough

Exponential backoff (1s, 2s, 4s, 8s, 16s...) spreads load over time. But when subscribers are synchronized, every retry wave still lands at the same instant, just less often.

The fix is **jitter**: add randomness to each delay. Instead of `delay = base * 2^attempt`, use `delay = random(0, base * 2^attempt)`. This breaks the synchronization across subscribers.

Without jitter, 1,000 subscribers still slam the endpoint at each backoff step. With jitter, those 1,000 retries spread across the full interval.

## Two-phase retry: the production pattern

Pure exponential backoff has a problem: delays grow too fast. After 10 attempts at base 2s, you are waiting 17 minutes between retries. After 15 attempts, 9 hours. You either cap the delay (reinventing two-phase) or accept huge gaps.

Two-phase retry splits the schedule into two stages. The **fast phase** uses short, increasing delays to catch transient failures within seconds. The **slow phase** switches to fixed-interval retries over hours or days for longer outages.

This maps to how failures actually play out: most resolve in seconds (a restart, a deploy rollout), while some take hours (provider outage, DNS propagation).

```mermaid
stateDiagram-v2
    [*] --> Deliver
    Deliver --> Success : 2xx
    Deliver --> FastRetry : transient failure
    FastRetry --> Deliver : delay 10s → 5min
    FastRetry --> SlowRetry : fast retries exhausted
    SlowRetry --> Deliver : delay 1h fixed
    SlowRetry --> Failed : slow retries exhausted
    Success --> [*]
    Failed --> [*]

    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef danger fill:#fee2e2,stroke:#f87171,color:#7f1d1d
```

## How Hook0 retries

Hook0 does not expose per-phase or per-subscription retry tuning. It applies one fixed schedule of increasing delays to every failed delivery:

| Failed attempts so far | Delay before next attempt |
|---|---|
| 1 | 3 seconds |
| 2 | 10 seconds |
| 3 | 3 minutes |
| 4 | 30 minutes |
| 5 | 1 hour |
| 6 | 3 hours |
| 7 | 5 hours |
| 8 and beyond | 10 hours |

The seconds-apart attempts at the start catch the transient failures that resolve in moments, like container restarts, deploy rollouts, or brief network partitions. The hours-apart attempts later cover longer outages, such as provider downtime or DNS propagation, without hammering an endpoint that is coming back. From the eighth retry on, the delay holds at 10 hours.

Two limits bound the schedule, whichever is reached first: `MAX_RETRIES` (default 25) and `MAX_RETRY_WINDOW` (default 8 days). Both are set on the output worker, not per subscription. With the defaults, a failing delivery is retried up to 25 times across roughly 8 days. The full reference is in [Webhook retry logic](/explanation/webhook-retry-logic).

## Further reading

- [Webhook retry logic](/explanation/webhook-retry-logic) -- how Hook0 schedules and bounds retries
- [Request attempts](/concepts/request-attempts) -- how each delivery attempt is tracked
- [Monitor webhook performance](/how-to-guides/monitor-webhook-performance) -- delivery rates and retry patterns

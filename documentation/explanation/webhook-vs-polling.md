---
title: "Webhook vs Polling: When Real-Time Beats Repeated Requests"
description: "Latency numbers, cost math, and a decision framework for choosing webhooks, polling, or a hybrid of both."
keywords: [webhook vs polling, webhooks vs api polling, real-time notifications, webhook latency, polling cost, push vs pull, polling fallback, hybrid webhook polling]
faqItems:
  - question: "Is a webhook better than polling?"
    answer: >-
      When the source supports webhooks, yes for most cases. A webhook sends
      one HTTP request per change, usually in under a second. Polling every 60
      seconds adds 30 seconds of delay on average and spends almost every call
      on an empty response. Polling still wins when the source only offers a
      pull API, when a cron script is all you need, or when you want a full
      snapshot of state on every run.
  - question: "When do you have to poll instead of using a webhook?"
    answer: >-
      Poll when the source has no webhooks, when your consumer cannot expose a
      public HTTPS endpoint (a laptop, a locked-down network), or when the
      sender drops deliveries without retrying. Polling is also the simpler
      choice for detecting deletions and reconciling full state, because each
      run returns a complete snapshot.
  - question: "What is the hybrid webhook and polling pattern?"
    answer: >-
      Webhooks carry changes in real time, and a low-frequency polling job
      (hourly, for example) pulls full state and diffs it against what you
      already processed. Both paths feed the same handler, which skips
      duplicates with an idempotency key. Use it when reliability matters more
      than latency: the poll catches anything a network blip, a sender outage
      or a misconfiguration made the webhook miss.
  - question: "At what volume does polling get too expensive?"
    answer: >-
      Cost grows with the number of resources you poll and the interval, not
      with the number of changes. Polling one list endpoint once a minute is
      1,440 calls a day, which most APIs accept even for a few hundred changes
      a day. Polling 10,000 resources individually once a minute is 14.4
      million calls a day to find roughly 2,400 changes, while webhooks send
      about 2,400 requests.
  - question: "Do I still need polling if the sender retries webhooks?"
    answer: >-
      Often not. When the sender retries failed deliveries over several hours,
      a short outage on your side does not lose events. Hook0 retries on a
      fixed schedule from 3 seconds up to 10-hour intervals and lets you replay
      any event, so a polling fallback becomes a safety net for long outages
      rather than a requirement.
---

# Webhook vs Polling: A Decision Framework

## Short version

**Polling** means your code asks "anything new?" on a timer. Works fine, but wastes calls at scale.

**Webhooks** mean the source sends you an HTTP request when something changes. Faster and cheaper, though you need an endpoint to receive them.

If your source supports webhooks, use webhooks. If it does not, poll. If reliability matters, do both (see [hybrid pattern](#the-hybrid-pattern) below).

## Latency comparison

| Scenario | Polling (60s interval) | Polling (5s interval) | Webhook |
|----------|:---:|:---:|:---:|
| Average notification delay | **30s** | **2.5s** | **< 1s** |
| Worst-case delay | 60s | 5s | Network RTT |
| Delay scales with | Poll interval | Poll interval | Nothing |

Polling latency = `interval / 2` on average. Shrinking the interval improves latency but multiplies API calls linearly.

## Cost comparison

Consider 10,000 resources you need to monitor for changes, where ~1% change per hour:

| Metric | Polling (60s) | Polling (5s) | Webhook |
|--------|:---:|:---:|:---:|
| API calls / day | **14.4M** | **172.8M** | **~2,400** |
| Calls that found a change | ~2,400 | ~2,400 | ~2,400 |
| Wasted calls / day | 14,397,600 | 172,797,600 | 0 |
| Wasted call ratio | 99.98% | 99.999% | 0% |

With webhooks, every HTTP request carries a real change. With polling, you pay for every empty response in API quota, bandwidth, server CPU, and rate-limit headroom.

## When polling wins

Webhooks are not always the right call.

If the source API only supports pull, you poll. No other option there. Same if the webhook sender has no retry logic and silently drops deliveries -- polling becomes your safety net.

For simple scripts, a cron job polling once per minute is 10 lines of code. Standing up an HTTPS endpoint with signature verification takes more work. And when you need full-state consistency, polling gives you a snapshot each time, which makes it easier to detect deletions or reconcile state.

## When webhooks win

When users expect instant feedback (payment received, order shipped, CI build done), webhooks deliver it. The cost gap matters too: 14.4M vs 2,400 API calls per day adds up across resources and environments.

Both sides benefit from lower load. The source stops serving millions of empty responses, and your system stops parsing them. On top of that, a single [event](/concepts/events) can fan out to multiple [subscriptions](/concepts/subscriptions) at once. Polling would need separate logic per consumer.

## The hybrid pattern

The most reliable setup is webhook-first with a polling fallback:

```mermaid
flowchart LR
    SRC["Source system"]:::external -->|Push event| WH["Webhook receiver"]:::hook0
    WH --> PROC["Process change"]:::processing
    POLL["Polling job (hourly)"]:::customer -->|Pull full state| SRC
    POLL --> DIFF["Diff against last known state"]:::processing
    DIFF -->|Missed changes?| PROC

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764
```

The webhook path delivers changes in real-time. Then a low-frequency polling job runs (say, hourly) to catch anything the webhook missed -- network blip, sender outage, misconfiguration. Both paths feed the same processor, which uses an idempotency key to skip duplicates.

Hook0's [retry logic](/explanation/webhook-retry-logic) does something similar on the sending side: if a delivery fails, it retries with increasing delays, so you may not need the polling fallback at all.

## How Hook0 handles this

Hook0 wraps the "webhook-first" pattern into something you can run in production without building retry queues, signature verification, or delivery monitoring from scratch.

It accepts [events](/concepts/events) from your application via API, routes them to subscriber endpoints based on [subscriptions](/concepts/subscriptions), and retries failed deliveries on a [fixed escalating schedule](/explanation/webhook-retry-logic). You also get delivery logs, latency metrics, and replay.

The [getting started tutorial](/tutorials/getting-started) walks through setup.

## FAQ

### Is a webhook better than polling?

When the source supports webhooks, yes for most cases. A webhook sends one HTTP request per change, usually in under a second. Polling every 60 seconds adds 30 seconds of delay on average and spends almost every call on an empty response. Polling still wins when the source only offers a pull API, when a cron script is all you need, or when you want a full snapshot of state on every run.

### When do you have to poll instead of using a webhook?

Poll when the source has no webhooks, when your consumer cannot expose a public HTTPS endpoint (a laptop, a locked-down network), or when the sender drops deliveries without retrying. Polling is also the simpler choice for detecting deletions and reconciling full state, because each run returns a complete snapshot.

### What is the hybrid webhook and polling pattern?

Webhooks carry changes in real time, and a low-frequency polling job (hourly, for example) pulls full state and diffs it against what you already processed. Both paths feed the same handler, which skips duplicates with an idempotency key. Use it when reliability matters more than latency: the poll catches anything a network blip, a sender outage or a misconfiguration made the webhook miss.

### At what volume does polling get too expensive?

Cost grows with the number of resources you poll and the interval, not with the number of changes. Polling one list endpoint once a minute is 1,440 calls a day, which most APIs accept even for a few hundred changes a day. Polling 10,000 resources individually once a minute is 14.4 million calls a day to find roughly 2,400 changes, while webhooks send about 2,400 requests.

### Do I still need polling if the sender retries webhooks?

Often not. When the sender retries failed deliveries over several hours, a short outage on your side does not lose events. Hook0 retries on a [fixed schedule](/explanation/webhook-retry-logic) from 3 seconds up to 10-hour intervals and lets you replay any event, so a polling fallback becomes a safety net for long outages rather than a requirement.

## Further reading

- [Event concepts](/concepts/events) -- what events are and how they flow through Hook0
- [Subscriptions](/concepts/subscriptions) -- routing rules that connect events to endpoints
- [Webhook retry logic](/explanation/webhook-retry-logic) -- how Hook0 handles failed deliveries
- [Event processing](/explanation/event-processing) -- the full event lifecycle

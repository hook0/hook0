---
title: "Where Do Webhooks Go After the Last Retry? Dead Letters in Hook0"
sidebar_label: "Dead letters after the last retry"
description: "Hook0 keeps no separate dead-letter queue. After 24 retries or 8 days the attempt stays failed, with its payload and response, and you replay the event."
keywords: [webhook dead letter queue, webhook dlq, failed webhooks, replay failed webhooks, retries exhausted, webhook replay, inbound vs outbound dead letter]
faqItems:
  - question: "Does Hook0 have a dead-letter queue for webhooks?"
    answer: >-
      When Hook0 stops retrying a delivery, the last request attempt stays in
      the failed status, next to the event payload and the response your
      endpoint sent back. That stored attempt plays the role of a dead letter,
      with no separate queue to manage. You list failed attempts through the API or the dashboard,
      fix the endpoint, then replay the event with POST /events/{event_id}/replay.
  - question: "When does Hook0 stop retrying a webhook?"
    answer: >-
      With the default settings, after 24 retries or 8 days, whichever comes
      first. Hook0 also stops early on a non-retryable error, such as a signature
      header that cannot be built, and when the subscription is disabled or
      deleted. Self-hosted deployments change the limits with MAX_RETRIES and
      MAX_RETRY_WINDOW.
  - question: "How do I replay webhooks after retries are exhausted?"
    answer: >-
      Call POST /events/{event_id}/replay with your application_id, or use the
      replay action on the event in the dashboard. Hook0 creates new request
      attempts for every active subscription that matches the event type and
      labels, so receivers should deduplicate on the event ID.
  - question: "What is the difference between a sender-side and a receiver-side dead letter?"
    answer: >-
      A sender-side dead letter is a delivery the sender gave up on, kept so the
      sender can replay it. That is what Hook0 stores for the webhooks you send.
      A receiver-side dead letter is an incoming webhook your own app failed to
      process. Tools such as Requeue keep those for you by sitting between the
      provider and your app. Hook0 does not offer an inbound dead-letter inbox in
      production.
---

# Where do webhooks go after the last retry?

A dead-letter queue (DLQ) holds the messages a system gave up delivering, so someone can look at them and send them again. Hook0 does not move exhausted deliveries into a separate queue. The last [request attempt](/concepts/request-attempts) stays in the `failed` status, next to the [event](/concepts/events) payload and the response your endpoint sent back, and you replay the event once the endpoint is fixed.

This page covers when a delivery becomes a dead letter, what Hook0 keeps, how to find failed deliveries and replay them, and how sender-side dead letters differ from the receiver-side inboxes some teams run for the webhooks they receive.

## When a delivery becomes a dead letter

Hook0 retries a failed delivery on a fixed schedule, from 3 seconds to 10 hours apart (see [Retry logic](webhook-retry-logic.md)). It stops in four cases:

| Condition | Default | What Hook0 does |
|---|---|---|
| Retry count reached | `MAX_RETRIES` = 24 | No new attempt; the last one stays `failed` |
| Retry window exceeded | `MAX_RETRY_WINDOW` = 8 days | No new attempt; the last one stays `failed` |
| Non-retryable error | Always on | Stops at once, for example when a signature header cannot be built |
| Subscription disabled or deleted | Always on | Stops retrying for that subscription |

Self-hosted deployments tune the first two limits on the output worker (see [Configuration](/reference/configuration)). Hook0 Cloud uses the defaults.

## What Hook0 keeps

For each delivery, Hook0 stores one request attempt per try. A failed attempt carries:

- the event it belongs to and the subscription it targeted;
- the retry count and the time it was created, picked up and marked failed;
- the HTTP status code your endpoint returned, or the network error name when there was no response;
- the response body and headers, which you fetch separately when debugging.

The event payload is stored once, on the event. Replaying reuses it, so you never rebuild a payload by hand.

## Find failed deliveries

In the dashboard, open the application's events and request attempts; failed attempts show their status code and error. Through the API, list request attempts for an application and keep the failed ones:

```bash
curl -s "$HOOK0_API/request_attempts/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '[.[] | select(.status.type == "failed") | {event_id, subscription_id, retry_count, http_response_status}]'
```

Narrow the list with `event_id`, `subscription_id`, `min_created_at` and `max_created_at`, or with `event.event_type_names`. Results are paginated through the `Link` header. [Debug failed webhooks](../how-to-guides/debug-failed-webhooks.md) walks through reading the status code, error name and response of each attempt.

## Replay a dead letter

Once the endpoint is fixed, replay the event:

```bash
curl -X POST "$HOOK0_API/events/$EVENT_ID/replay" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"application_id": "'"$APP_ID"'"}'
```

Replay sends the event again to every active subscription that matches its type and labels, with a fresh retry schedule. Two consequences follow:

- a subscription that already received the event gets it a second time, so receivers deduplicate on the event ID (see [Idempotency](webhook-retry-logic.md#idempotency));
- when the subscription is degraded or recovering, the new attempts are created paused and released at a capped rate, so a replay after an outage does not flood an endpoint that just came back (see [Subscription health](webhook-retry-logic.md)).

## Sender-side and receiver-side dead letters

The term "dead letter" covers two different jobs, on opposite ends of the same webhook.

| | Sender-side (Hook0) | Receiver-side inbox |
|---|---|---|
| Who gave up | The sender, after its retries | Your app, while handling an incoming webhook |
| What is kept | Your event and each failed delivery attempt | The raw incoming request and the error |
| Who replays | You, to your customers' endpoints | You, into your own app |
| Typical tools | Hook0, or a queue DLQ you build | A relay inbox such as Requeue, or an inbound gateway such as Hookdeck |

Hook0 covers the sender side, meaning the webhooks you send to your users. It does not offer a production inbox for the webhooks you receive from Stripe, Clerk or other providers. The [Hook0 vs Requeue](https://www.hook0.com/hook0-vs-requeue) and [Hook0 vs Hookdeck](https://www.hook0.com/hook0-vs-hookdeck) comparisons cover the receiver-side tools.

## FAQ

### Does Hook0 have a dead-letter queue for webhooks?

When Hook0 stops retrying a delivery, the last request attempt stays in the failed status, next to the event payload and the response your endpoint sent back. That stored attempt plays the role of a dead letter, with no separate queue to manage. You list failed attempts through the API or the dashboard, fix the endpoint, then replay the event with `POST /events/{event_id}/replay`.

### When does Hook0 stop retrying a webhook?

With the default settings, after 24 retries or 8 days, whichever comes first. Hook0 also stops early on a non-retryable error, such as a signature header that cannot be built, and when the subscription is disabled or deleted. Self-hosted deployments change the limits with `MAX_RETRIES` and `MAX_RETRY_WINDOW`.

### How do I replay webhooks after retries are exhausted?

Call `POST /events/{event_id}/replay` with your `application_id`, or use the replay action on the event in the dashboard. Hook0 creates new request attempts for every active subscription that matches the event type and labels, so receivers should deduplicate on the event ID.

### What is the difference between a sender-side and a receiver-side dead letter?

A sender-side dead letter is a delivery the sender gave up on, kept so the sender can replay it. That is what Hook0 stores for the webhooks you send. A receiver-side dead letter is an incoming webhook your own app failed to process. Tools such as Requeue keep those for you by sitting between the provider and your app. Hook0 does not offer an inbound dead-letter inbox in production.

## Further reading

- [Retry logic](webhook-retry-logic.md): the exact schedule and limits
- [Webhook delivery guarantees](webhook-delivery-guarantees.md): at-least-once delivery and ordering
- [Debug failed webhooks](../how-to-guides/debug-failed-webhooks.md): read status codes, errors and responses
- [Request attempts](/concepts/request-attempts): the attempt model and its statuses

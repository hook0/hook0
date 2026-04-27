---
title: Pricing & billing
description: Hook0 pricing plans, overage billing, and how to monitor your usage
keywords: [pricing, billing, overage, quotas, events, consumption, dashboard]
---

# Pricing & billing

Hook0 offers several plans to match your needs. For comprehensive details, consult the [official pricing page](https://www.hook0.com/pricing).

## Plans overview

| Feature | Developer (Free) | Startup (€59/mo) | Pro (€190/mo) | Enterprise |
|---------|:-:|:-:|:-:|:-:|
| Events per day | 100 | 30,000 | 100,000 | Custom |
| Data retention | 7 days | 14 days | 30 days | Custom |
| Applications | 1 | 1 | Unlimited | Custom |
| Developers | 1 | 25 | Unlimited | Custom |
| When quota exceeded | **Blocked** (HTTP 429) | **Billed as overage** | **Billed as overage** | Custom |
| Overage rate | N/A | €0.003/event | €0.0001/event | Custom |

## Quota behavior by plan

### Free (Developer) plan

When you exceed your daily event quota, new [events](../concepts/events.md) are **rejected** with an HTTP 429 status code (`TooManyEventsToday`). The quota resets daily.

### Paid plans (Startup, Pro)

On paid plans, event ingestion is **never interrupted** when the daily quota is exceeded. Extra events are billed as overage at the per-event rate for your plan. This ensures uninterrupted service delivery for customers building products on top of Hook0.

## Billing cycle

- Subscriptions are billed monthly (or annually with a discount on the Pro plan).
- Overage charges are calculated at the end of each billing period and invoiced with the following period's invoice.
- Invoices are generated via Stripe and available in your Stripe billing portal.

## Monitoring your usage

The **Organization Dashboard** in the [Hook0 app](https://app.hook0.com) shows your event consumption for the current day and past days. The dashboard displays event volumes but does not show prices — for billing details, check your Stripe billing portal.

Hook0 also sends email notifications when your daily consumption approaches the quota threshold.

## Special Discount Program

Charities, not-for-profit organizations, and educational institutions receive a 25% discount off of list prices.

To obtain this discount, users must:
1. Select a plan through the Organization page in the Hook0 app
2. Contact Hook0 support to apply the discount

If you are uncertain about your eligibility, please reach out to us directly for clarification.

## Next steps

- See [Quotas and limits](/concepts/applications#quotas-and-limits) for application-level quota details
- See [Error codes](/reference/error-codes#toomanyeventstoday) for the `TooManyEventsToday` error reference
- Learn about [Client error handling](/how-to-guides/client-error-handling) for handling quota errors in your code

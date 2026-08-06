---
title: Architecture
description: Hook0 instance architecture and components
---

# Architecture

A Hook0 instance is composed of multiple parts. Let's explain what they are used for!

## Hook0 API

The central part of the system, a Rust web application.

**Dependencies:**
- PostgreSQL database

## Hook0 UI

A Vue.js front-end web application.

**Dependencies:**
- Hook0 API

## Hook0 Output Worker

Responsible for actually calling users' webhooks and gathering responses. A Rust application that does not need to accept incoming connections.

**Dependencies:**
- PostgreSQL database used by Hook0 API

:::note

There can be multiple instances of Hook0 Output Worker, work would be shared between each of them.

:::

## Email your instance sends on its own

Besides the transactional messages a user's own action triggers (address verification, password reset), a self-hosted instance sends one automated sequence: a short onboarding drip to accounts that verified their address but never ingested an event, at J+1, J+3 and J+7 after sign-up. It stops as soon as the account sends its first event, only ever targets sign-ups younger than 30 days, and every message carries a one-click opt-out link.

It is on by default because an instance that never nudges a stuck user is the more common failure. If you would rather Hook0 never emailed your users, set `ENABLE_REACTIVATION_EMAILS=false` — see [Configuration](../reference/configuration.md#reactivation) for that and the other knobs (cadence, per-pass cap, CTA URLs).

## What's Next?

- [Bare Metal](bare-metal.md)
- [Docker Compose](docker-compose.md)
- [Kubernetes](kubernetes.md)
- [AWS](aws.md)
- [Master API Key](master-api-key.md)

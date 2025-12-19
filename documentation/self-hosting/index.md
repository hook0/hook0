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

## What's Next?

- [Bare Metal](bare-metal.md)
- [Docker Compose](docker-compose.md)
- [Kubernetes](kubernetes.md)
- [AWS](aws.md)
- [Master API Key](master-api-key.md)

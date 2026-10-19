---
title: Docker Compose
description: Quick local setup using Docker Compose
---

import DevOnlyWarning from './_dev-only-warning.mdx';

# Docker Compose

This guide covers setting up Hook0 using Docker Compose for local development.

<DevOnlyWarning />

## Prerequisites

- Docker installed on your machine
- Docker Compose installed

## Setup

Clone the repository and run Docker Compose:

```bash
git clone https://gitlab.com/hook0/hook0.git
cd hook0
docker compose -f docker-compose.yaml up --build --detach
```

The initial build requires significant time.

## User Registration

After deployment, create an account using the registration endpoint:

```bash
curl http://localhost:8081/api/v1/register \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "your@email.com",
    "first_name": "Your",
    "last_name": "Name",
    "password": "your-secure-password"
  }'
```

## Email Verification

Hook0 sends a verification email before you can log in. In local development, emails are captured by Mailpit:

1. Open Mailpit at `http://localhost:8025`
2. Find the verification email sent to your address
3. Click the verification link in the email

## Access

Clicking the verification link signs you in and lands you on the dashboard at
`http://localhost:8001`. Later on, log in there directly.

## Receiving Webhooks on Localhost

This stack delivers straight to `localhost`, so a subscription can target a handler on your machine.
If you only need to receive webhooks on a local port, from Hook0 Cloud or any other sender, you do
not need the stack at all: [`hook0 listen 3000`](../reference/cli.mdx) prints a public URL and
forwards each request to port 3000. See
[how the Hook0 CLI compares to an ngrok tunnel for local webhook testing](https://www.hook0.com/hook0-vs-ngrok).

## Data Storage

Docker volumes include:

| Volume | Path | Purpose |
|--------|------|---------|
| postgres-data | /var/lib/postgresql/data | Hook0 database |

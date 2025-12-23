---
title: Master API Key
description: Instance-wide authentication for self-hosted Hook0
---

# Master API Key

A **master API key** is a special authentication token for self-hosted instances. It can be used to authenticate almost any API call instance-wide.

## Scope and Limitations

The master API key works across the entire instance, unlike JWTs and application secrets that are scoped to organizations and applications respectively.

:::warning

The only endpoint that cannot be used with the master API key is the events ingestion endpoint.

:::

## Enabling the Feature

To activate this functionality, set the `MASTER_API_KEY` environment variable with a UUID value when running Hook0's API:

```bash
export MASTER_API_KEY=your-uuid-here
./hook0-api
```

The feature is disabled by default.

## Usage

API calls using the master API key require this HTTP header format:

```
Authorization: Bearer [token]
```

Example:

```bash
curl -X GET "https://your-hook0-instance.com/api/v1/organizations" \
  -H "Authorization: Bearer your-master-api-key"
```

## Security Warning

:::danger

The master API key functions as basically a huge backdoor. Anyone that has access to it can have full control of your Hook0 instance, regardless of internal organizations/permissions.

:::

We strongly recommend:

1. Only enable this feature during initial setup
2. Disable it immediately after completing setup by unsetting the environment variable
3. Never share the master API key
4. Rotate the key if you suspect it has been compromised

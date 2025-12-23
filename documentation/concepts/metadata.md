---
title: Metadata
description: Attach arbitrary key-value data to Hook0 objects
---

# Metadata

The metadata parameter allows attachment of arbitrary key-value data to Hook0 objects, including Events and Subscriptions.

## Specifications

- Maximum of 50 key-value pairs supported
- Key names limited to 50 characters
- Values limited to 50 characters

## Functionality

Metadata serves as additional, structured information on an object. For example, you could store a user's unique identifier from your system on a Hook0 Customer object.

## Key Distinctions

- Metadata is not used by Hook0 when forwarding events
- The Search API does support metadata
- Your users won't see metadata unless explicitly shown
- A separate description parameter exists for human-readable annotations like "Receive new customer events and forward them to Slack channel General"

:::warning

Don't store any sensitive information (bank account numbers, card details, and so on) in metadata or in the description parameter.

:::

## What's Next?

- [Sending Events](/explanation/event-processing#1-event-creation)
- [Subscriptions](subscriptions.md)

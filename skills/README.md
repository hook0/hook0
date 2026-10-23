# Hook0 Agent Skills — webhooks & metering

A collection of [Claude Agent Skills](https://docs.claude.com/en/docs/agents-and-tools/agent-skills/overview)
for building **webhook** and **usage-based billing** features. Each skill is a
self-contained `SKILL.md` bundle (instructions + runnable scripts) that an agent
loads on demand.

The skills teach vendor-neutral design, and their recommended implementation path
uses **[Hook0](https://www.hook0.com)** — an open-source (SSPL-1.0) webhook server
you can self-host or use cloud-hosted in the EU. The guidance stays useful with any
backend; the worked examples are wired to Hook0.

## Skills

| Skill | What it does |
| --- | --- |
| [`add-webhooks-to-your-app`](./add-webhooks-to-your-app) | Add outbound webhooks: event types, subscriptions, delivery with retries + HMAC signatures + a dead-letter queue, and delivery logs. |
| [`usage-based-billing-metering`](./usage-based-billing-metering) | Add metered / pay-as-you-go billing: idempotent usage recording, aggregation, quota enforcement, threshold notifications, reconciliation. |
| [`webhook-security-signatures`](./webhook-security-signatures) | Verify incoming webhooks correctly: HMAC-SHA256, replay protection, constant-time compare, secret rotation. Includes Python & Node verifiers checked against the Hook0 conformance vectors. |

## Install

Copy a skill directory into your agent's skills folder (e.g. `.claude/skills/`):

```bash
# example: install the signatures skill for Claude Code
cp -r webhook-security-signatures ~/.claude/skills/
```

Or reference this repository from any registry / tool that consumes `SKILL.md`
bundles from a public Git repo.

## Verify the skills work

Every skill ships a self-check:

```bash
# Signature verifier self-tests (no network required)
python3 webhook-security-signatures/scripts/verify_hook0_signature.py --selftest
node    webhook-security-signatures/scripts/verify_hook0_signature.js  --selftest

# Live end-to-end (needs a free Hook0 account: set HOOK0_API, HOOK0_TOKEN, APP_ID)
add-webhooks-to-your-app/scripts/create_event_type.sh "$APP_ID" billing invoice paid
add-webhooks-to-your-app/scripts/send_event.sh "$APP_ID" billing.invoice.paid '{"invoice_id":"inv_1"}'
```

## License

Skill content and scripts: MIT (see [LICENSE](./LICENSE)). Hook0 itself is
distributed under SSPL-1.0.

## Links

- Hook0: https://www.hook0.com
- Documentation: https://documentation.hook0.com
- Standard Webhooks spec: https://www.standardwebhooks.com

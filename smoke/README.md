# The live smoke

Every client this repository generates, held against a Hook0 that is really running.

## What this is for

Each client ships a suite that is exhaustive about behaviour — how many times a send is retried,
how long it waits, what it does with a truncated answer, what it does with a hostile header — and
each of those suites gets that by talking to a server it starts itself on loopback. That is the
right way to test those things: a real API cannot be made to stall, truncate or lie on demand.

What none of those suites can ask is whether the client can talk to Hook0 **at all**. A client can
pass everything it has and still fail on first contact, because four things are only true of a real
instance:

- an application secret the API minted is accepted as authentication;
- a problem document has the shape the client reads a problem out of;
- a duplicated ingestion is refused as `EventAlreadyIngested`, in the answer the client matches on;
- a signature computed by the **output worker** verifies against the subscription secret.

The fourth is the one worth the whole exercise. A suite that signs with the module it is testing and
verifies with the same module proves the two halves agree; it does not prove either agrees with
Hook0.

So: one flow per language, not a second copy of each suite. Running twelve exhaustive suites against
a real server would be slow, flaky, and would prove nothing the loopback ones already prove.

## What one run does

1. Brings up the repository's own `docker-compose.yaml`, unmodified, and waits for the API to
   answer — under a deadline, with a clear refusal if it never does.
2. Provisions the way a user does: registers an account, reads the verification email out of the
   instance's Mailpit, opens the session that verification returns, and creates an application, an
   application secret, an event type and a subscription. Nothing is inserted into the database and
   no master key is used.
3. Points that subscription at a socket the harness opens, sends one event, and keeps the webhook
   the output worker delivers exactly as it arrived — the body's bytes, the header names and values
   in order, and the `X-Hook0-Signature` the worker computed.
4. Runs every client's smoke. Each one sends an event under an identifier its own client minted,
   confirms the API accepted it, sends the same event again under that identifier and confirms the
   conflict comes back naming `EventAlreadyIngested`, and verifies the caught delivery with its own
   verification code.
5. Takes the stack down, volumes included, on every path out — the failing ones too.

## Running it

```sh
cargo run --manifest-path smoke/Cargo.toml
```

Docker has to be reachable by the user running it, and the ten other toolchains have to be on the
`PATH`. The Lua rock needs `luasocket` and `luasec` for the interpreter it runs under:

```sh
luarocks install --local --lua-version=5.4 luasocket
luarocks install --local --lua-version=5.4 luasec
eval "$(luarocks --lua-version=5.4 path)"
```

The output worker has to be able to reach a socket the harness opens on the host, since that is
where the webhook whose signature is verified is delivered. On a machine whose firewall drops
traffic arriving from the Docker bridge — `ufw` does by default — the run gets as far as
provisioning and then refuses with *the output worker delivered no webhook*. Allowing that one
bridge is what unblocks it:

```sh
sudo ufw allow in on "br-$(docker network ls --filter label=com.docker.compose.project=hook0-smoke --format '{{.ID}}')" from 172.16.0.0/12 proto tcp
```

The one question that needs no Docker at all — is every client the generator declares paired with a
smoke — is a plain test:

```sh
cargo test --manifest-path smoke/Cargo.toml --test discovery
```

## Attaching to a stack somebody else started

Setting `HOOK0_SMOKE_API_URL` makes the harness attach instead of composing, which is what CI does:
that runner has no Docker, so the job starts the same processes as GitLab services and points the
harness at them. `HOOK0_SMOKE_MAILPIT_URL` is required in that mode, since Mailpit is not on
loopback there; `HOOK0_SMOKE_RECEIVER_HOST` says what address the output worker reaches the harness
on, and defaults to `127.0.0.1`.

## How the set of clients is decided

It is not decided here. `hook0_sdkgen::targets::targets()` is the registry of what this repository
generates, and a smoke is a directory under `languages/` named after a target. A target with no
directory **fails the run**; a directory naming no target fails it too, because nothing would run
it. A twelfth client added tomorrow is smoked without anyone editing this crate.

What runs a smoke is data belonging to the smoke rather than a table in the orchestrator: each
directory carries a `smoke.toml` naming its command, so no language's toolchain is spelled out in
Rust.

## What each smoke is handed

| Variable | What it is |
|---|---|
| `HOOK0_API_URL` | The base URL of the API, ending in `/api/v1` |
| `HOOK0_APPLICATION_ID` | The application created for this run |
| `HOOK0_TOKEN` | An application secret the API minted |
| `HOOK0_EVENT_TYPE` | An event type the application declares |
| `HOOK0_DELIVERY` | A directory holding the webhook that was really delivered |

The delivery is one plain file per part — `signature`, `body`, `headers`, `secret`, `tolerance` —
rather than one document, because eleven of the twelve languages would otherwise spend most of a
smoke on a JSON parser. `headers` is one `name: value` per line, names lowercased, values as
delivered, in the order they arrived.

The tolerance is wide. The delivery is caught once, at the start, and the last language to verify it
does so after every toolchain ahead of it has compiled. What is under test here is a code over bytes
the server produced; the width of the acceptance window is what the shared conformance corpus
exercises, in every client, against vectors with a moment pinned in them.

## What it found on its first run

Two clients do not pass, and neither failure is in this harness. Both are the kind of defect that
only a real instance can show, which is the argument for the whole exercise.

**The TypeScript client drops the API's base path.** It resolves the endpoint with
`new URL('event', this.apiUrl)`, and relative resolution against a base whose path has no trailing
slash discards the last segment: given `https://app.hook0.com/api/v1` — the base URL the sibling
SDKs' READMEs all spell without a trailing slash — it posts to `/api/event`. The 404 that comes back
carries no body, so the failure is reported as `Sending event … failed: Error`, with nothing said
about what went wrong. Every other client accepts the base URL either way.

**The MCP server discards the identifier of the problem it was told.** `Hook0McpClient` reduces an
error answer to `message`, `error` or `detail`, in that order, and Hook0's problem document carries
the stable name under `id`. So a duplicated ingestion reaches the assistant as
`API error (409): This event was previously ingested…` — English prose, under the generic
`internal_error` code. There is no machine-readable way to tell that conflict from any other, which
is exactly what an assistant deciding whether to retry needs.

## The one client that does less

`mcp` sends and reports the conflict like the others, over the stdio transport the server ships
rather than as a linked library — that is its public interface. It verifies no signature, and the
absence is the answer rather than a gap: its tools are generated from the API's OpenAPI document,
which declares no operation for verifying a webhook. There is no consumer half to hold a
server-produced signature against.

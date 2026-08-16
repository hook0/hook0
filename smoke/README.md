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
where the webhook whose signature is verified is delivered. On a machine whose firewall drops what
arrives from the Docker bridge — `ufw` does, by default, and says nothing about it — no delivery
can ever arrive. The run finds that out for itself, before it provisions anything, and prints the
rule that opens it, naming the subnet it just read off the daemon:

```
sudo ufw allow from 172.18.0.0/16 to any comment 'docker bridge'
```

A rule on the *subnet* rather than on the bridge interface, deliberately. Compose creates and
destroys its network on every run, and the interface it makes — `br-<network id>` — is named after
an id that changes each time, so a rule pinned to the interface works for exactly one run and then
silently stops applying. That is not a hypothesis: it is how this was first found.

## When it refuses

Every refusal names what it expected. Three are worth knowing in advance:

- **the receiver cannot be reached from inside the stack** — the firewall above, found by a request
  over exactly the path the delivery takes, before an account is even registered. This check is the
  difference between one sentence and a two-minute silence with four equally plausible causes;
- **no webhook within the deadline** — the deadline is [derived](src/worker.rs) from the output
  worker's own defaults rather than picked, and the refusal shows the arithmetic and the last lines
  the worker logged while it was being waited for;
- **a process stopped on its own** — reported before anything is waited for, because a worker that
  exited at startup otherwise arrives, much later, as a webhook that never came.

Setting `HOOK0_SMOKE_KEEP` leaves a *failed* run's stack standing, with the command to take it down
by hand. A run that passed takes it down whatever is set. It is for the questions nobody thought of
in advance, which are the only ones a removed container cannot answer.

The one question that needs no Docker at all — is every client the generator declares paired with a
smoke — is a plain test:

```sh
cargo test --manifest-path smoke/Cargo.toml --test discovery
```

## The two ways up, and what is shared between them

Setting `HOOK0_SMOKE_BINARIES` to a directory holding `hook0-api` and `hook0-output-worker` makes
the harness start those two as processes instead of composing containers. That is what CI does: its
runner has no Docker, so Postgres and Mailpit are GitLab services and the binaries come from
`api-integration-tests.build`.

Only the way the two are started differs. **Both are started by the harness, with the environment
read out of `docker-compose.yaml`** — so the CI job declares no `DATABASE_URL`, no
`BISCUIT_PRIVATE_KEY` and no port, and there is no second copy of the configuration to drift.
Everything after that point is one code path: the readiness wait, the provisioning, the delivery,
the smokes, the teardown. Nothing downstream can tell which way the stack came up.

The order is shared too, and not by coincidence: the compose file declares `output-worker` as
depending on `api` being healthy, because the API is what runs the migrations. The process path
waits for the API to answer before starting the worker, for the same reason — started alongside, the
worker asks for a table that does not exist yet and exits, leaving a stack that answers, provisions,
and then never delivers. A process this harness started that stops on its own is reported as that,
before anything is waited for.

Three things genuinely cannot be shared, and each is commented where it happens:

| | Containers | Processes |
|---|---|---|
| `DISABLE_SERVING_WEBAPP` | from the API's image (`api/Dockerfile`) | set by the harness, same value |
| Mailpit | `127.0.0.1`, the published port | the alias the compose file names, same port |
| The receiver | the Compose network's gateway | `127.0.0.1` |
| Proving the receiver is reachable | a `curl` in the `api` container | a request from this host |
| What the worker said | `docker compose logs` | already on this run's own output |

The reachability check runs from the `api` service rather than from the worker, and that is not a
guess about what the image carries: the compose file's own health check for `api` **is** a `curl`,
and it sits on the same bridge network as the worker, which is what makes their route to this host
the same route.

One more difference is worth knowing about because it is not configuration: the containers are
built from source by the image build, while CI reuses the binary `api-integration-tests.build`
produced with `--no-default-features -F application-secret-compatibility`. Both carry the feature
the harness needs; the bytes are not the same bytes.

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

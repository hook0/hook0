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
- a signature computed by the **output worker** verifies against the subscription secret;
- every operation the client generates really round-trips: the request it composes is one Hook0
  answers, and that answer is one it can read into the value — or the problem — it declares;
- every model the client generates is really decodable: the field names it was generated from are
  the field names Hook0 sends.

The fourth is the one worth the whole exercise. A suite that signs with the module it is testing and
verifies with the same module proves the two halves agree; it does not prove either agrees with
Hook0.

So: one flow per language, not a second copy of each suite. Running twelve exhaustive suites against
a real server would be slow, flaky, and would prove nothing the loopback ones already prove.

**The fifth is not that, and the sentence above is not an argument against it.** Every client's
loopback suite already drives its whole generated surface — each endpoint method, each model, the
whole problem catalogue. Every one of them drives it against an API the suite itself writes. That
API is the test author's belief about how Hook0 answers, and nothing checks the belief: a generated
model whose field names do not match what the instance really returns passes all twelve suites and
fails on a consumer's first call. No suite can hold its own fake against the real thing. So the
surface is driven here too — once per language, against a Hook0 that is really running, with each
operation reported and the set of reports held to the set of operations the API document declares.

What is not repeated here is what those suites are exhaustive about: retry counts, backoff timing,
truncated answers, hostile headers. Those need a server that can be made to stall, truncate and lie
on demand, and a real instance is exactly the thing that cannot.

## What one run does

1. Brings up the repository's own `docker-compose.yaml`, unmodified, and waits for the API to
   answer — under a deadline, with a clear refusal if it never does.
2. Provisions the way a user does: registers an account, reads the verification email out of the
   instance's Mailpit, opens the session that verification returns, and creates an application, an
   application secret, an event type and a subscription. Nothing is inserted into the database and
   no master key is used.
3. Points that subscription at a socket the harness opens, sends one event, and keeps the webhook
   the output worker delivers exactly as it arrived — the body's bytes, the header names and values
   in order, and the `X-Hook0-Signature` the worker computed. That delivery is caught **once** and
   every language verifies the same bytes.
4. Waits, once, for the two things only the instance can produce: the attempt and the response the
   output worker left behind, and the per-day counts the API refreshes on a cycle of its own. No
   language waits for either.
5. Runs every client's smoke, each against **an application of its own**, created out of that same
   session immediately before the smoke starts. Each one sends an event under an identifier its own
   client minted, confirms the API accepted it, sends the same event again under that identifier
   and confirms the conflict comes back naming `EventAlreadyIngested`, verifies the caught delivery
   with its own verification code, and drives every operation the API document declares — reporting
   each one on its own output.
6. Holds each language to two bijections — the operations it drove against the ones the document
   declares, and the model types it decoded against the ones an operation answers — and refuses
   naming everything that is on one side and not the other.
7. Takes the stack down, volumes included, on every path out — the failing ones too.

An application each rather than one between twelve, because deleting an application, an event type
and a subscription are operations the clients declare and therefore operations a smoke has to drive.
With one shared application the first language to delete it would take the eleven behind it with
it.

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

The questions that need no Docker at all — is every client the generator declares paired with a
smoke, and does the bijection hold what it says it holds — are plain tests:

```sh
cargo test --manifest-path smoke/Cargo.toml
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
Rust. The same file is where a smoke says it does not drive the generated surface yet — see below.


## What every smoke must print

Two kinds of line, on its own output. One per operation it drove:

```
exercised <operationId> accepted
exercised <operationId> refused:<problemId>
```

and one per generated model type it decoded out of a real answer:

```
decoded <ModelName>
```

- `accepted` — the API answered a success **and the client decoded it into the generated value**.
- `refused:<problemId>` — the API answered a problem document **and the client decoded it as that
  problem**, named as the document names it (`Forbidden`, `NotFound`, `EventAlreadyIngested`, …).

Both count, and that is deliberate. An operation the smoke's credential is not authorised for still
proves the client composed the request, reached the API and read the answer, which is the whole
question being asked. What does not count is anything else: a transport failure, an answer the
client cannot decode, a body naming a problem the client does not know. None of those say the client
and the instance agree on anything, so the smoke says so and exits non-zero rather than reporting a
line.

`<operationId>` is the id the API document gives the operation — `events.ingest`,
`subscriptions.delete` — not a method name the language happens to spell it with. `<ModelName>` is
likewise the name the document declares the type under — `RequestAttemptStatus`,
`ApplicationInfoOnboardingSteps` — not the name the language spells it with, so that the harness
holds twelve languages to one set.

A `decoded` line is printed where the flow **holds the decoded value**, not where it issued the
request. In Rust that is enforced by taking the value: a type that stops being part of an answer
stops compiling rather than going on being reported. Do whatever the equivalent is in your language,
and if there is none, at least print the line from the same scope as the value.

The harness reads those lines off both of a smoke's streams while writing them through, so a smoke
that hangs still shows what it managed to say. It then holds the language to a **bijection** with the
operations its own client is generated from: the API document, narrowed through `hook0_sdkgen` by
the tag that target's entry in the generator's registry selects — `PUBLIC_TAG` for the eleven SDKs,
and its own for the MCP server, which is generated from a different set. It refuses, naming what is
wrong:

- an operation the document declares that the language never drove — **every** one of them is named,
  not the first;
- a report naming an operation id the document does not declare, which is what a typo looks like
  from here: left alone it would satisfy nothing while the operation it was meant to name was
  reported as never driven;
- one operation reported twice with two different outcomes, which is two call sites disagreeing
  about which operation they drove. Twice with the *same* outcome is a flow that read a list before
  and after it changed something, and is fine;
- `refused:RateLimited`, which is the one problem a report may not name — see below.

That bijection is what stops these flows rotting. An operation the API grows fails every language
until somebody drives it.

### And a second bijection, on the models

The operation bijection alone is satisfiable by a client that can decode nothing. Every operation
could come back refused — an unauthorised credential refuses in exactly the shape an authorised one
accepts — and thirty-six `refused:` lines would pass while not one generated model had been parsed
out of a real answer. That is the bug class this whole exercise exists to catch, so the models are
held to a bijection of their own.

The set is derived the same way the operations are, and narrowed twice, both times by the document
rather than by anything written here. A model is expected iff **an operation answers it**, reached
from a success body **through fields the document says are always there**. Which leaves out, by
derivation and not by a list:

- types only a request body carries. Nobody ever decodes one. What would catch a wrong field name
  there is the operation that sends it being refused, which the other bijection already holds;
- types reachable only through a field the document marks optional. Those are decoded by an
  instance that populates them and by no other, so holding every run to them would be holding it to
  a configuration rather than to a client.

The refusals mirror the operation ones: every model an operation answers and the language never
decoded is named, and a `decoded` line naming something the generator does not emit at all is
refused as the typo it is. A model the generator emits but no operation answers is *allowed* to be
reported — a smoke on an instance that populates one of the optional ones is right to say so.

### Writing one

`languages/rust/src/main.rs` is the one to read first. It is written as client code a consumer could
learn from rather than as a test — there is no shared scenario language for the twelve to interpret,
because an interpreter would make every one of these unreadable and would itself be a new untested
surface — and it orders the flow the way an application would: it creates what it needs, reads and
lists it, updates it, and destroys last.

Five things the instance will teach whoever writes the next one, in the order they bite:

- **organization-scoped operations are refused.** `applications.list`, `applications.create`,
  `serviceToken.*` and `events_per_day.list_for_organization` belong to the organization, and an
  application secret is scoped to one application. They come back `Forbidden`, which is a report.
- **never revoke the secret you are holding.** `applicationSecrets.delete` succeeds on the secret
  the flow is authenticating with, and every request after it is refused. Mint a second one, drive
  the operations against that, and delete that one.
- **delete the application last.** The same applies to it, and more so: the secret stops
  authenticating the moment its application is gone.
- **the reads that need the worker use the seeded ids.** `requestAttempts.get` and `response.get`
  address rows the output worker wrote, in `HOOK0_SEEDED_APPLICATION_ID`, with the organization
  credential. Asking for one in your own application answers nothing, and waiting for one is the
  harness's job, done once.
- **keep one genuine refusal.** With both credentials in hand almost everything succeeds, and a
  flow whose every answer is a success never finds out whether the client can read a failure. The
  rust flow drives `applications.create` with the *application* secret for exactly that reason, and
  reads the problem document that comes back.

### The one problem that is not a report

Hook0 paces callers per credential: with the defaults this stack runs, a token gets a burst of
twenty requests and then about ten a second. A flow driving three dozen operations one after another
runs into that, and what comes back is `RateLimited`, 429, with a `Retry-After` header.

That answer is not a report, and the harness refuses a smoke that makes one of it. Every other
refusal is a round trip worth having — the request was composed, the instance answered *about that
operation*, and the client read it. This one says the instance never looked. A language that let it
through would report every operation as driven while proving nothing about any of them, which is the
one way this whole exercise can pass and mean nothing.

So a smoke waits out the delay the answer names and sends the request again, bounded both ways: a
number of tries and a longest wait. The clean place for it is the transport the client is handed,
which is the same place a real application would put it — every call site stays a call site.

### Saying a smoke does not drive the surface yet

A smoke that has not been written to drive the surface says so in its own manifest:

```toml
drives_surface = false
```

Absent means it does, which is the state every smoke ends up in and the only state a client added
tomorrow can be in. There is no list of ported languages anywhere in this crate: the line lives
beside the code that would drive the surface, so porting a language and saying it has been ported
are the same edit, and the key disappears from the tree entirely once the last language is ported.

Both halves are held, which is what makes it impossible to leave a language quietly untested:

- a manifest saying `drives_surface = false` while the smoke reports operations is refused, so a
  language cannot be ported without the line coming out;
- a smoke reporting nothing while its manifest says nothing is refused, so once the line is out
  there is no way back to silence. Deleting the reports from a ported language fails the run rather
  than passing it.

## What each smoke is handed

| Variable | What it is |
|---|---|
| `HOOK0_API_URL` | The base URL of the API, ending in `/api/v1` |
| `HOOK0_ORGANIZATION_ID` | The organization every application of the run belongs to |
| `HOOK0_APPLICATION_ID` | An application created for **this language alone**, moments before it ran |
| `HOOK0_TOKEN` | An application secret the API minted, for that application |
| `HOOK0_SERVICE_TOKEN` | An organization-scoped credential, for what an application secret may not reach |
| `HOOK0_EVENT_TYPE` | An event type that application declares |
| `HOOK0_SEEDED_APPLICATION_ID` | Another application of the same organization, which the instance has already delivered a webhook for |
| `HOOK0_REQUEST_ATTEMPT_ID` | A delivery attempt of that application, which the output worker has finished with |
| `HOOK0_RESPONSE_ID` | The response the worker wrote for that attempt |
| `HOOK0_DELIVERY` | A directory holding the webhook that was really delivered |

`HOOK0_APPLICATION_ID` and `HOOK0_TOKEN` differ from language to language and every other variable
is the same for all of them. A smoke may do whatever it likes to its own application, deletion
included; nothing it does there is visible to another language. `HOOK0_EVENT_TYPE` names the same
event type in every application, so it is the same string whichever smoke reads it.

`HOOK0_API_URL` ends in `/api/v1` because that is what the hand-written half of every client is
built with. The generated half is not built with it: the paths it composes already carry `/api/v1`,
since the API document's own server URL is the bare origin. Whichever of the two a language points
its generated layer at, it points it at the origin — `HOOK0_API_URL` with that path taken off. A
language that passes the variable through unchanged reaches `/api/v1/api/v1/...` and gets a 404 back
for every operation.

**Two credentials, because the API takes two.** `HOOK0_TOKEN` is an application secret, scoped to
one application; `HOOK0_SERVICE_TOKEN` is scoped to the organization. Several operations the
document declares are the organization's — `applications.list`, every `serviceToken.*`,
`events_per_day.list_for_organization` — and no application secret can perform them. A smoke holding
only the first could report them, but only ever as refusals, and the types they answer would go
undecoded by every language. Both are bearer tokens and the generated layer is credential-agnostic,
so this is one more transport, not a second client.

**Three ids for what only the instance can produce.** A request attempt exists once the output
worker has picked one up, and a response once it has finished with it; the per-day counts come out
of a view the API refreshes on a cycle of its own. None of that is a client's business, and no
language should be waiting on a worker inside its own flow. So the harness waits once, in the
application it caught the shared delivery from, and hands `HOOK0_SEEDED_APPLICATION_ID` with the
attempt and response ids. Read them with the organization credential — they belong to another
application of the same organization. `events_per_day.list_for_organization` is likewise non-empty
by the time any smoke runs, and is where `EventsPerDayEntry` gets decoded; a smoke's own
`events_per_day.list_for_application` answers an empty list, which is an answer and still a report.

The delivery is one plain file per part — `signature`, `body`, `headers`, `secret`, `tolerance` —
rather than one document, because eleven of the twelve languages would otherwise spend most of a
smoke on a JSON parser. `headers` is one `name: value` per line, names lowercased, values as
delivered, in the order they arrived.

The tolerance is wide. The delivery is caught once, at the start, and the last language to verify it
does so after every toolchain ahead of it has compiled. What is under test here is a code over bytes
the server produced; the width of the acceptance window is what the shared conformance corpus
exercises, in every client, against vectors with a moment pinned in them.

## What it found on its first run

Two clients failed, and neither failure was in this harness. Both are the kind of defect that only a
real instance can show, which is the argument for the whole exercise. Both are fixed, and each is
now held by a case in its own client's suite as well as by the run below.

**The TypeScript client dropped the API's base path.** It resolved the endpoint with
`new URL('event', this.apiUrl)`, and relative resolution against a base whose path has no trailing
slash discards the last segment: given `https://app.hook0.com/api/v1` — the base URL the sibling
SDKs' READMEs all spell without a trailing slash — it posted to `/api/event`. Every other client
accepts the base URL either way. The 404 that came back carried no body, and what the caller was
handed for it was `Sending event … failed: Error`, which names neither what was reached nor what
came back. Both are fixed: the base is given its trailing slash once, when the client is built, and
a refused send names the status, the problem the answer identified, and what that answer said.

**The MCP server discarded the identifier of the problem it was told.** `Hook0McpClient` reduced an
error answer to `message`, `error` or `detail`, in that order, and Hook0's problem document carries
the stable name under `id`. So a duplicated ingestion reached the assistant as
`API error (409): This event was previously ingested…` — English prose, with no machine-readable way
to tell that conflict from any other, which is exactly what an assistant deciding whether to retry
needs. The name is read out of the document and reported ahead of the prose now.

## The one client that does less

`mcp` sends and reports the conflict like the others, over the stdio transport the server ships
rather than as a linked library — that is its public interface. It verifies no signature, and the
absence is the answer rather than a gap: its tools are generated from the API's OpenAPI document,
which declares no operation for verifying a webhook. There is no consumer half to hold a
server-produced signature against.

Its surface is smaller too, and that is not this file's decision either: the registry says the MCP
target selects a different tag out of the same document, so the bijection holds it to the operations
its own tools are generated from rather than to the SDKs'.

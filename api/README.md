Hook0 API
=========

# Setup dev-env

- Spawn a local postgresql server checkout [database](../database)
- Setup database url in `.env`
- Start API

```bash
cargo run --bin api
```

## Updating queries

sqlx-cli is required to update prepared statements

```bash
cargo install sqlx-cli
cargo sqlx prepare
```

# Adding or changing an endpoint

Two mechanics apply to every handler of this crate. Both are enforced by tests, so ignoring them
fails the pipeline.

## The `public` tag

An operation reaches the generated clients only when its `#[api_v2_operation(...)]` carries the
`public` tag:

```rust
#[api_v2_operation(
    summary = "List subscriptions",
    operation_id = "subscriptions.list",
    tags("Subscriptions Management", "mcp", "public")
)]
```

Nothing is public by default and no list is maintained anywhere: the tag written on the handler is
the only thing that decides. A handler added without it is served by the API and exists for no
client.

**When to write it.** Write it on the operations a Hook0 user integrates against — sending events,
declaring event types, managing subscriptions and their secrets, reading delivery history. Leave it
off the control plane the dashboard drives for itself: signing in, registering, inviting a member,
billing. `mcp` is the same opt-in for the tools the MCP server exposes; the two sets overlap without
being identical.

**What it commits you to.** Publication and permanence. A tagged operation is what the generated
clients hand to their users: the MCP server's tools, and the SDKs generated for twelve languages.
Its name, its parameters and its responses become a public contract — renaming it, dropping a field
or narrowing a response afterwards breaks code we do not control. Tagging is cheap up to the first
release and expensive after it, so when the shape of an endpoint is still moving, ship it untagged
and tag it once it has settled.

A tagged operation also has to:

- spell its `operation_id` as `entity.verb`: one dot, both halves readable as identifiers
  (`subscriptions.list`, `applicationSecrets.create`). Generators split on that dot to name an
  entity and one of its methods, so an identifier spelled any other way breaks generation for every
  language at once, and a test rejects it.
- use a canonical verb whenever one fits: `list`, `get`, `create`, `update`, `delete` (`load` reads
  as `get`, `remove` as `delete`). Those render identically in every target. Any other verb is kept
  and becomes a method under the name you gave it — `events.replay` stays `replay` — so reach for
  one only when none of the five describes what the operation does.
- answer every error with the `Problem` schema, which is what generated clients turn into a typed
  error.

Those checks live in `src/handlers/public_surface.rs` and run against the document the application
actually serves, never against a list kept by hand.

## The OpenAPI snapshot

`openapi.snapshot.json` is the committed copy of that surface. The MCP server and the SDK generator
read it at build time instead of reaching a running instance, so it has to match what the
application serves. A test compares the two, and any change to a handler — signature, summary,
responses, tags — makes it fail.

The failure names what moved, then the command that adopts it:

```bash
UPDATE_OPENAPI_SNAPSHOT=1 cargo test -p hook0-api openapi_snapshot
```

Run it and commit the rewritten snapshot along with your change. Read the report before you do: it
is the only place an unintended change to the public surface shows up. An operation you did not mean
to touch, or one you did not mean to publish, is a defect in the handler and not in the snapshot.

Only `public` and `mcp` operations enter the snapshot, so work on the control plane leaves it
untouched.

### LICENSE

Hook0 is free and the source is available. Versions are published under
the [Server Side Public License (SSPL) v1](./LICENSE.txt).

The license allows the free right to use, modify, create derivative works, and redistribute, with three simple
limitations:

- You may not provide the products to others as a managed service
- You may not circumvent the license key functionality or remove/obscure features protected by license keys
- You may not remove or obscure any licensing, copyright, or other notices

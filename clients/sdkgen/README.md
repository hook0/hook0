# hook0-sdkgen

Reads a Hook0 OpenAPI snapshot and derives the entity model the Hook0 targets are built from. It
is the only place in the repository that parses the snapshot: targets read the model, never the
document.

Entities and their methods come out of the `entity.verb` convention the operation ids already
follow, so the API surface is never written down twice:

```
applications.list    ─┐
applications.create   ├─►  entity `applications`, methods list / create / get / update / delete
applications.get      │
applications.update   │
applications.delete  ─┘
```

`list`, `get` (or `load`), `create`, `update` and `delete` (or `remove`) are the canonical verbs
every target renders the same way. Any other verb stays a method named after the verb the spec
gives it — `events.replay` stays `replay` — and is never dropped. An operation the convention
cannot place at all, such as one without a separator or without an `operationId`, lands in
`EntityModel::unconventional` with the reason it could not be placed.

Each target names the tag it is built from: `PUBLIC_TAG` for the SDKs, `MCP_TAG` for the MCP
server. When the snapshot carries that tag, the model is narrowed to the operations marked with it;
otherwise every operation of the snapshot is kept. The tag itself is written on the handler, in
`api/src/handlers`; `api/README.md` says when it is warranted and what it commits the API to.

## Using it

```rust
use hook0_sdkgen::{EntityModel, Limits, PUBLIC_TAG, Snapshot};

let limits = Limits::default();
let path = std::path::Path::new("openapi.snapshot.json");
let snapshot = Snapshot::from_path(path, PUBLIC_TAG, &limits)?;
let model = EntityModel::from_snapshot(&snapshot, &limits)?;

for entity in model.entities() {
    println!("{} — {} methods", entity.name, entity.methods.len());
}
# Ok::<(), hook0_sdkgen::Error>(())
```

## Targets

What the generator writes is a registry of plain values — `targets::targets()` — rather than a set
of entry points. Each entry names itself, the tag it selects out of the snapshot, where it lands
relative to the root of the repository, what it owns there, how its language spells things, and the
one function turning the model into files.

Today it carries twelve entries, the eleven client libraries and `mcp`. The MCP one is the odd one
out. `hook0-mcp` is published to crates.io with no copy of the snapshot beside it, so its tool table
is a committed source file, `clients/mcp/src/server/generated.rs`, that this crate writes and the
server merely compiles. The MCP crate depends on nothing here, not even at build time.

Every target is emitted and checked through one driver, `tests/targets.rs`:

```
cargo test -p hook0-sdkgen sdk_targets                 # check every target
UPDATE_SDK=1 cargo test -p hook0-sdkgen sdk_targets    # rewrite every target
UPDATE_SDK=mcp cargo test -p hook0-sdkgen sdk_targets  # rewrite one, still check the rest
```

The rewritten files are committed. Without the variable the same test fails, naming the command for
each target that drifted — all of them at once, so adopting one change cannot bury another — and a
surface that never reached a generated tree stops in CI rather than at a release. What `UPDATE_SDK`
accepts is the registry itself: a target added to it is addressable with no second list to update.

## Bounds

Every input is bounded by `Limits`: snapshot size, reference depth, operation count, entity count,
methods per entity, parameters per operation and identifier length. A snapshot crossing a ceiling
is rejected with the count it reached and the ceiling it crossed — nothing is trimmed down to fit.

## Tests

`cargo test -p hook0-sdkgen` runs the suites below, all black box:

- `tests/model.rs`: the committed snapshot of the Hook0 API plus documents written for a single
  behaviour each;
- `tests/model_properties.rs`: property-based checks on idempotence, on conservation of the
  operations, and on the ceilings, with past failures replayed from `proptest-regressions/`;
- `tests/targets.rs`: the emission driver — the drift guard over every committed target, that the
  registry is what `UPDATE_SDK` accepts, and that two emissions of a target yield the same tree;
- `tests/mcp_tools.rs`: the shape of the MCP tool surface — one tool per selected operation, an
  idempotent emission, and the `mcp` tag selecting what the `public` tag leaves out;
- `tests/mcp_tools_properties.rs`: property-based checks that every selected operation yields
  exactly one tool, that tool names never collide, that re-emission writes the same bytes, and that
  an operation no tool could carry stops the emission;
- `tests/snapshot_fuzz.rs`: a bounded fuzzing run over the snapshot parser and the emitter,
  replaying the corpus in `tests/__fuzz__/snapshot/corpus/` before drawing random inputs. Longer
  campaigns go through `cargo bolero test snapshot`.

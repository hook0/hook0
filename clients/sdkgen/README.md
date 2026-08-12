# hook0-sdkgen

Reads a Hook0 OpenAPI snapshot and derives the entity model the Hook0 SDKs are built from.

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

When the snapshot carries the `public` tag, the model is narrowed to the operations marked with it;
otherwise every operation of the snapshot is kept.

## Using it

```rust
use hook0_sdkgen::{EntityModel, Limits, Snapshot};

let limits = Limits::default();
let snapshot = Snapshot::from_path(std::path::Path::new("openapi.snapshot.json"), &limits)?;
let model = EntityModel::from_snapshot(&snapshot, &limits)?;

for entity in model.entities() {
    println!("{} — {} methods", entity.name, entity.methods.len());
}
# Ok::<(), hook0_sdkgen::Error>(())
```

## Bounds

Every input is bounded by `Limits`: snapshot size, reference depth, operation count, entity count,
methods per entity, parameters per operation and identifier length. A snapshot crossing a ceiling
is rejected with the count it reached and the ceiling it crossed — nothing is trimmed down to fit.

## Tests

`cargo test -p hook0-sdkgen` runs three suites, all black box:

- `tests/model.rs`: the committed snapshot of the Hook0 API plus documents written for a single
  behaviour each;
- `tests/model_properties.rs`: property-based checks on idempotence, on conservation of the
  operations, and on the ceilings, with past failures replayed from `proptest-regressions/`;
- `tests/snapshot_fuzz.rs`: a bounded fuzzing run over the snapshot parser, replaying the corpus in
  `tests/__fuzz__/snapshot/corpus/` before drawing random inputs. Longer campaigns go through
  `cargo bolero test snapshot`.

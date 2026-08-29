# hook0-dashboard-examples

Reads the example every client keeps beside itself and writes the one artefact the dashboard's
"Send an event" screen renders from, `frontend/src/generated/sdkExamples.ts`. It is the only place
that knows both halves. The screen knows no language, and no client knows the screen.

The screen used to carry its snippets as strings typed into a `.vue` file. One of them had stopped
compiling, passing a `&None` where the client declares an `Option<&Uuid>`, and nothing anywhere
could have noticed, because a snippet written over there is backed by nothing. Moving it next to the
client puts it under the job that already builds that client, so a renamed method turns that job red
on the day it happens rather than turning up in a reader's editor weeks later.

## What a target owes

Every entry of `hook0_sdkgen::targets::targets()` declaring `Contract::Whole`, which is every
generated client library and which leaves the MCP server out by its own declaration, owes three
files under `clients/<target>/examples/`:

```
dashboard_send.<ext>      what the screen shows under "Send the event"
dashboard_verify.<ext>    what it shows under "Verify a delivery"
dashboard.toml            what a string substitution cannot express
```

A target without them fails. Any of the three sitting where no target is fails, whichever one it is
and whatever extension it carries. The guard runs both ways and admits no exception, so the next
client is picked up by whoever adds it rather than by whoever remembers this crate exists.

## Markers

The two files are read for a region rather than whole, so anything they need only in order to be
checked stays out of what a reader sees:

```rust
// hook0:snippet:begin        what is displayed starts here
// hook0:label:begin          one rendering of one label
// hook0:label:end
// hook0:snippet:end
```

A marker is located by position inside a line rather than by the line it sits on. `rustfmt` moves an
end marker up onto the end of the line of code before it, and no formatter can be talked out of what
it does. Nothing else may share a line with one, and each appears exactly once.

`hook0:label` belongs to `dashboard_send` alone. The region it delimits is repeated once per label
the form carries and joined with the separator the manifest declares, so it sits *inside* its
container, and no label at all leaves that container validly empty. It carries no trailing separator
of its own; a formatter that adds one back has it stripped.

## Substitution

The screen substitutes seven words and does nothing else. Five belong outside the label region and
two belong inside it:

```
__HOOK0_API_URL__  __HOOK0_APPLICATION_ID__  __HOOK0_TOKEN__  __HOOK0_EVENT_TYPE__  __HOOK0_PAYLOAD__
__HOOK0_LABEL_KEY__  __HOOK0_LABEL_VALUE__
```

They are string literals, which is what lets a file full of them compile. A word shaped like a
marker that is not one of the seven is refused where it is found. Nothing would replace it, so it
would reach a reader exactly as it is written, in code they were told to copy.

Both snippets are also held to what they must say. A `dashboard_verify` reads
`HOOK0_SUBSCRIPTION_SECRET` itself, because the screen has no secret to give it and an argument
arriving from nowhere answers the reader nothing.

It reads it loudly, too. Verification asks nothing of the key it is handed, and hashes the delivery
against whatever it was given before reporting a mismatch. So a snippet that quietly reads an empty
secret when nobody exported the variable refuses every genuine delivery as a forged one, and the
reader, told the signature is bad, goes looking at their own signing code.

One property is what stops that, and only that one. The line naming the variable carries no empty
string literal. An empty string spelled some other way goes past it, whether it is a named
constant, a defaulting call, or an assertion that a value is there. TypeScript's `!` was one of
those, and this does not catch it.

## The manifest

`dashboard.toml` holds what substituting a string cannot express, declared beside the code it
describes:

| Key | What it settles |
|-|-|
| `display_name` | how the language is named on screen, which neither the target name nor the package name is |
| `usage_share` | the published share of developers writing it, which is the order the languages are offered in, declared with its source; every manifest reads the same survey |
| `proof` | `compiled`, `type-checked` or `parsed`: how far the job carrying this client goes towards proving its example |
| `proves` | the command and the job that level rests on, written for whoever goes to check the level rather than take it; that it says something is held here, what it says is not |
| `examples_named_in`, `examples_named_by` | the file whose build configuration puts `examples/` under that job, and the lines in it that do it; held against the file itself, so deleting one fails here rather than quietly leaving the level standing over a directory nothing opens |
| `examples_swept_by` | the command instead, for a language where nothing names the directory because the command reads a tree the examples are in; there is no line to delete and none to hold, and saying so is what keeps the difference visible |
| `snippet_also_needs` | what a reader installs or wires beyond the package itself before the send snippet builds, appended to the install block; the one key a manifest may leave out |
| `label_separator` | what joins two rendered labels, carrying the comma the region must not |
| `[string]` | `open`, `close`, and the ordered `escape` rules turning a reader's payload into a literal, backslash first, or escaping the quotes escapes the backslashes those just introduced |

Exactly one of `examples_named_in` and `examples_swept_by` is declared. Neither, and nothing says
what puts the examples under the job; both, and nothing says which of the two the level rests on.

`snippet_also_needs` is the one a manifest may omit, and nine of the eleven do. What installs a
package is derived from the registry serving it, and neither Rust's sending API being async-only nor
Zig's module having to be handed to a build is a fact about a registry — so the language says it,
and what it says is appended to the install block rather than left for the reader to discover at the
block below. Declaring it and leaving it empty is refused, as is a `__HOOK0_*__` word inside it: the
install block is substituted through the same table as the two snippets, so a marker there puts what
the reader typed into a command they were told to run.

Only the `escape` rules reach the screen. `open` and `close` are read by the property test, which
needs to know what a literal is delimited by in order to assert that nothing a reader types closes
one. They look unused from the renderer and are not. `proof` and `proves` reach it no further:
they are read, held, and left for whoever opens the manifest, since what a panel tells a reader
about how far a snippet was proven is a decision about the product rather than about this crate.

## Using it

```
cargo run -p hook0-dashboard-examples     # rewrite the artefact
cargo test -p hook0-dashboard-examples    # check it, and everything above
```

The rewritten file is committed, and the check refuses a committed artefact that disagrees with what
a regeneration produces. A copy of the truth that has stopped being true is the defect this crate
exists to remove, so introducing one at its own edge would be absurd. `ci/pre-release-sdk.sh`
regenerates before it commits, because the artefact carries every package's version and a release
moves it.

Nothing here has a job of its own. It rides `client.rust.check`, which already runs on every example
in the tree.

## Bounds

Every input is bounded and nothing is trimmed down to fit: 64 SDKs, 128 kB per example, 64 kB per
manifest, 16 kB per region, 64 characters of display name, 16 of separator, 16 per delimiter, 32
escape rules of 16 characters, 512 characters of `proves`, 512 of what a snippet needs beyond its
package, 8 lines naming what puts the examples under their job at 512 characters apiece, 1024 of the
source a share is read off, and a share between 0 and 100. A file crossing a ceiling is refused,
naming what it reached and what it crossed.

## Tests

`cargo test -p hook0-dashboard-examples` runs three suites, all black box:

- `tests/parity.rs`: the registry against the tree in both directions, the artefact against the
  registry, the freshness of what is committed, that every language declares how far its proof
  goes, and that the configuration putting its examples under the job it names still does;
- `tests/shape.rs`: what a region comes out as, and what it is refused for. The formatter moving a
  marker, the trailing separator put back, the indentation a region sits on, a misspelled marker, a
  label outside its region, a verify that hides where its secret comes from, and one that falls
  back to an empty secret rather than raising;
- `tests/escaping.rs`: property-based checks that no value a reader can type closes the literal it
  lands in, in every language shown, with past failures replayed from `proptest-regressions/`.

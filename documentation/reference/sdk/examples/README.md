# Proving the examples on these pages

Every fenced example of `documentation/reference/sdk` is assembled against the client it claims to
use and handed to that language's own toolchain. What runs it is the `documentation-examples`
crate, the job is `documentation.examples` in the pipeline, and this directory is the half a
page's author touches. Nothing under it is published: the Docusaurus build excludes it, because it
is addressed to whoever edits a snippet rather than to whoever reads one.

A snippet is written for a reader: it omits the imports, assumes a client is already built, and
names a token without saying where it came from. Rather than pad the page with that, each snippet
says which **harness region** it drops into, and the region — beside the page, in the page's own
language — is the rest of the file. Nothing about an example lives in the checker.

## What a page declares

Its front matter names the generated target it documents:

```yaml
sdkTarget: go
```

The value is a target of the generator's registry (`hook0_sdkgen::targets::targets()`), or `none`
for a page that documents no single client. A page claiming a target and showing no example of
that target's language is refused, and so is a page claiming a target the generator does not
produce.

Every fence whose language is a target of the registry is an example, and says where it goes:

````markdown
```go example=upsert
created, err := client.UpsertEventTypes(ctx, []string{"billing.invoice.paid"})
```
````

A fence in one of those languages with no `example=` is refused, because it would otherwise be the
one snippet nobody proved. A fence carrying `example=` in a language no target claims is refused
too. Fences in every other language — `bash`, `toml`, `json`, `xml` — are prose and are left
alone.

## What a language declares

One directory per target, named after it:

```
examples/go/
  examples.toml     how an example is proven, and what that proves
  harness.go        the rest of the file, once per region
  template.go.mod   the project every example of this language is built inside
```

Everything that is neither `examples.toml` nor `harness.*` is the project scaffold: it is copied
next to the assembled examples, with `{{client}}` replaced by the absolute path of the client this
target emits into and `{{repository}}` by the repository root.

A scaffold file whose name starts with `template.` loses that prefix when it is copied, so
`template.go.mod` becomes the project's `go.mod`. That exists for the files a substitution leaves
invalid until it happens: `{{client}}` sits where a module path belongs, so a committed file named
`go.mod` is one no Go-aware tool can read — including the dependency scanner, which reads every
`go.mod` in the tree and fails the build on one it cannot parse. Use the prefix only where the
committed form would be read by something as the real thing; a `tsconfig.json` or a `Cargo.toml`
carrying a placeholder is nobody's manifest and needs nothing.

### `examples.toml`

```toml
proof = "compiled"                # compiled | type-checked | parsed
proves = "assembled against clients/go and built; a renamed method fails"
path = "{{name}}/main.go"         # where one assembled example lands
timeout_seconds = 300             # the budget one command gets
run = [["go", "build", "./..."]]  # commands run once, in the project root

[environment]
GOPROXY = "off"
```

`proof` is the claim the report prints, and it must not overstate what the command does:

| Level | What it means |
|-------|---------------|
| `compiled` | Built against the real client. Catches a renamed method, a changed signature, a dropped field. |
| `type-checked` | Resolved against the real client's declared types without producing an artefact. Catches the same wherever the client's types reach. |
| `parsed` | Read by the language's own parser. Catches syntax, and nothing about the client. |

`each = [["ruby", "-c", "{{file}}"]]` runs once per assembled example instead, for the tools that
take one file at a time. `{{name}}` is the example's name (`go_07`), `{{Name}}` the same in the
casing a class wants (`Go07`); both are also substituted inside the harness, which is how a
language whose file names a type gets a class name that matches its file.

Anything a language needs that the image does not carry is installed by the language itself, as a
command of its own `run` — the way the Lua manifest pins luacheck and the Python one pins mypy. A
version pinned there is pinned once, beside the pages it governs, rather than a second time in the
pipeline.

### `harness.<ext>`

A region opens with `HARNESS <name>` inside a comment, closes with `END HARNESS`, and carries one
line whose only word is `EXAMPLE` — that is where the snippet is written, at that indentation.

```go
// HARNESS upsert
package {{name}}

import (
	"context"

	hook0 "github.com/hook0/hook0-go"
)

// The event types an application declares before it sends anything of them.
func declare(ctx context.Context, client *hook0.Client) ([]string, error) {
	EXAMPLE

	return created, err
}

// END HARNESS
```

The harness file is not compiled as it stands; each region becomes a file of its own. Anything
outside a region is a comment to whoever reads it next.

## Running it

```bash
cd documentation-examples
cargo run                      # every language
cargo run -- --only go         # one, while working on it
```

`--only` prints `PARTIAL RUN`, and the pipeline never passes it.

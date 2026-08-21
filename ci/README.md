# CI/CD Configuration

## Required CI Variables for Release Jobs

The manual release jobs (`release:patch`, `release:minor`, `release:major`) and SDK release jobs require the following CI/CD variables:

| Variable | Description | Required For |
|----------|-------------|--------------|
| `GITLAB_RELEASE_TOKEN` | Personal access token with `write_repository` scope | All releases |
| `NPM_TOKEN` | npm automation token, publish scope on `hook0-client` | SDK release (npm) |
| `CARGO_TOKEN` | crates.io API token, `publish-update` scope. Two crates go out under it: `hook0-client` on the SDK release and `hook0-mcp` on the MCP one. | SDK release (crates.io), MCP release |
| `PYPI_TOKEN` | PyPI API token scoped to the `hook0-client` project. Passed as the password with the user name `__token__`, which is what PyPI expects. | SDK release (PyPI) |
| `RUBYGEMS_API_KEY` | RubyGems API key with the `push_rubygem` scope. The gemspec sets `rubygems_mfa_required`, so the key has to be one `gem push` can use without prompting for an OTP — otherwise the job hangs until its timeout. | SDK release (RubyGems) |
| `NUGET_API_KEY` | nuget.org API key scoped to `Hook0.Client` with Push permission. NuGet keys expire (365 days maximum), so this one has a renewal date. | SDK release (NuGet) |
| `DOCKERHUB_USERNAME` | Docker Hub username | Container release |
| `DOCKERHUB_TOKEN` | Docker Hub access token | Container release |
| `GITHUB_USERNAME` | GitHub username for GHCR | Container release |
| `GITHUB_TOKEN` | GitHub PAT with `packages:write` scope | Container + GitHub release |
| `GITHUB_MIRROR_TOKEN` | GitHub PAT with `contents:write` on every `hook0/hook0-<language>` mirror. Held apart from `GITHUB_TOKEN` because it may write to repositories rather than to a package registry, and because the mirrors are the only thing it may write to. | SDK release (mirrors) |
| `PACKAGIST_USERNAME` | Packagist account name, passed as a query parameter to the update API. | SDK release (Packagist) |
| `PACKAGIST_API_TOKEN` | Packagist API token of that account. It only asks Packagist to re-read a repository it already knows, so registering `hook0/hook0-php` once by hand is what has to happen before the first release. | SDK release (Packagist) |

### Setup in GitLab

1. Go to https://gitlab.com/-/profile/personal_access_tokens
2. Create a token with `write_repository` scope for `GITLAB_RELEASE_TOKEN`
3. Go to Settings > CI/CD > Variables
4. Add each variable with its value
5. Mark all as "Protected" and "Masked"

## Release Process

1. Push commits to `master` branch
2. Go to CI/CD > Pipelines
3. On the master branch pipeline, find the manual jobs in `trigger-release` stage:
   - `release:patch` - Bumps patch version (2.0.0 → 2.0.1)
   - `release:minor` - Bumps minor version (2.0.0 → 2.1.0)
   - `release:major` - Bumps major version (2.0.0 → 3.0.0)
4. Click the play button on the desired release type
5. The job will:
   - Bump versions in api/Cargo.toml and output-worker/Cargo.toml (shared version)
   - Update frontend/package.json version
   - Generate/update CHANGELOG.md using git-cliff
   - Create a commit with the version bump
   - Create and push a git tag (e.g., v2.0.1)
6. The tag triggers the release pipeline which:
   - Validates all component versions match the tag
   - Builds binaries for linux/amd64
   - Builds frontend
   - Builds and pushes Docker images (amd64)
   - Creates GitHub release with all artifacts

## Pipeline Flow

```
master branch
    │
    ▼ (click release:patch/minor/major)
    │
pre-release.sh + cargo release → bump versions + changelog + commit + tag
    │
    ▼ (push tag)
    │
Tag pipeline triggered (vX.Y.Z)
    │
    ├─► release.verify-versions (check api, worker, frontend match tag)
    │
    ├─► release.build-api-amd64
    ├─► release.build-worker-amd64
    ├─► release.build-frontend
    ├─► release.extract-notes
    │
    ├─► release.containers (Docker images - amd64)
    │       ├─► GitLab Registry
    │       ├─► DockerHub
    │       └─► GHCR
    │
    └─► release.github (GitHub release with binaries)
```

## Artifacts

### Binaries
- `hook0-api-linux-amd64`
- `hook0-output-worker-linux-amd64`
- `frontend-dist.tar.gz`

### Docker Images
- `hook0/hook0-api:<version>` (DockerHub)
- `hook0/output-worker:<version>` (DockerHub)
- `ghcr.io/hook0/hook0-api:<version>` (GitHub Container Registry)
- `ghcr.io/hook0/output-worker:<version>` (GitHub Container Registry)
- `$CI_REGISTRY_IMAGE/hook0-api:<version>` (GitLab Registry)
- `$CI_REGISTRY_IMAGE/output-worker:<version>` (GitLab Registry)

## SDK Release Process

SDK releases are independent from main releases. Which clients they cover is not written down here
either — see below — and neither is which of those reach a registry today.

1. On the master branch pipeline, find the manual jobs in `trigger-release` stage:
   - `sdk-release:patch` - Bumps patch version (1.0.0 → 1.0.1)
   - `sdk-release:minor` - Bumps minor version (1.0.0 → 1.1.0)
   - `sdk-release:major` - Bumps major version (1.0.0 → 2.0.0)
2. Click the play button on the desired release type
3. The job will:
   - Read what the release covers out of the tree (`cargo run -p release-packages -- list`)
   - Write the new version into every SDK that declares one
   - Regenerate each package's `CHANGELOG.md`, scoped to its own directory
   - Create a commit with the version bump
   - Create and push a git tag (e.g., sdk-v1.0.1)
4. The tag triggers the SDK release pipeline, which checks the tree agrees with the tag and then runs every publish job the release has.

### What holds a release to being as big as it is

The bump is an argument — `patch`, `minor` or `major` — and the version it produces is the last
release plus one, so nothing in either script knows what actually happened since. What happened is
in the commit messages, and both `ci/pre-release.sh` and `ci/pre-release-sdk.sh` ask
`release-packages` to read them before writing anything:

```bash
cargo run -p release-packages -- required-bump patch 'mcp/v*' 'clients/mcp/**'
```

It reads every commit since the last tag of that shape that touches those paths, derives the
smallest release they allow — `major` for a `!` before the colon or a `BREAKING CHANGE:` footer,
`minor` for a `feat`, `patch` otherwise — and refuses a smaller one, naming the commits that demand
more. A larger one is allowed: deciding that a release is a major is a decision about what it means
to whoever installs it, and no commit log can overrule it. The answer to a refusal is therefore
either a bigger bump or a commit message that says what the commit did, since a change that breaks
the wire under a `chore:` subject is the case this exists to catch.

### What the release covers, and how it is decided

Nothing in `ci/` names an SDK. `ci/release-packages` reads the generator's registry
of targets (`hook0_sdkgen::targets::targets()`), walks up from where each target
lands to the one directory its package occupies under `clients/`, and reads the
package's name and version out of whatever manifest sits there. A target added to
the registry is therefore versioned, inventoried and held to the release tag by
machinery that already exists, and held to having somewhere to be published by
the section below; a target whose package cannot be established stops the release
naming itself and the paths it looked at, rather than shipping forever at
whatever version it was born with.

A package that owns a release flow of its own — `ci/release-<name>.gitlab-ci.yml`,
which today is only `clients/mcp` — has a tag of its own per ADR 0004 and is left
alone by the SDK release.

Run it against a working copy to see the inventory:

```bash
cargo run -p release-packages -- list
```

### What publishes a package, and what happens when nothing does

A publish job names the package directory it publishes in a `PUBLISHES` variable
and cds into it, so what a job says it publishes is what it does publish. That is
also how `ci/release-packages` knows a package has a publisher at all: it reads
every flow under `ci/`, and matches the declarations against the inventory in
both directions. A job naming a package the inventory has never heard of fails
the build, and so does a package no job names.

Not every package can reach a registry today. Every package in that position is
recorded in `ci/release-no-publish-job.toml`, one entry each, stating the registry
and the published name the inventory reports for it and the reason it has no job. The tool holds every entry to the package it
names, so a reason cannot be written about a package that does not exist, cannot
go on describing one that has been renamed or has moved ecosystems, and cannot
outlive the job that closes the gap. That file is where to find out what is
missing and what supplying it would take; there is deliberately no second copy of
those reasons here to fall out of step with them.

### The mirrors, and the three ecosystems that need one

GitLab holds the monorepo and runs this pipeline. Each SDK client also exists on
GitHub as a repository of its own — `github.com/hook0/hook0-<language>` — whose
root is that client. `sdk-release.mirrors` builds each one with `git subtree
split`, force-pushes the resulting commit onto the mirror's default branch and
pushes a plain `vX.Y.Z` tag beside it. A mirror is a derived artefact: nothing is
merged back from one, and nothing in this job writes to GitLab.

Which clients are mirrored is not written in the YAML either. `cargo run -p
release-packages -- mirrors` prints one line per client — the directory to split
and the repository to push it to — and the job loops over that, so a client added
to the generator's registry reaches a repository of its own without this file or
`ci/release-sdk.gitlab-ci.yml` changing. The repository name is derived from the
directory, and the run refuses past a ceiling of mirrors rather than discovering
one in a runner.

The repositories themselves are a prerequisite rather than something a release
creates. Nothing in this pipeline calls the GitHub API, and `git push` does not
bring a repository into being, so `sdk-release.mirrors` fails against a name that
is not there yet. Create all of them before the first release rather than most of
them: the job walks the list under `set -e`, and for Go, Packagist and Zig the
push is the publication, so stopping halfway leaves some languages published at a
version the others never reach, with the tag already spent. Each one is public,
since those three ecosystems all resolve a version by reading the repository
anonymously.

Three ecosystems can be published no other way, and their publish jobs are what
the mirrors are for:

- **Go** — a module is named by the address it is fetched at, so
  `clients/go/go.mod` declares `github.com/hook0/hook0-go` and the mirror is what
  answers there. `release-packages` refuses a release where the module path and
  the mirror have come apart, since `go get` of a path nothing serves resolves
  nothing while every job stays green. Publishing is asking `proxy.golang.org`
  for the version, which is what makes it fetch and cache the tag.
- **PHP** — Packagist accepts no uploads and reads no subdirectories. The mirror
  is the repository it is registered against; the release tells it to re-read the
  tags and then waits, bounded, for the version to appear.
- **Zig** — there is no registry at all. What is published is the mirror's tag
  archive, whose root is the package; the release fetches it back and holds it to
  carrying `build.zig.zon` at the version being released.

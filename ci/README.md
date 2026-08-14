# CI/CD Configuration

## Required CI Variables for Release Jobs

The manual release jobs (`release:patch`, `release:minor`, `release:major`) and SDK release jobs require the following CI/CD variables:

| Variable | Description | Required For |
|----------|-------------|--------------|
| `GITLAB_RELEASE_TOKEN` | Personal access token with `write_repository` scope | All releases |
| `NPM_TOKEN` | npm automation token, publish scope on `hook0-client` | SDK release (npm) |
| `CARGO_TOKEN` | crates.io API token, `publish-update` scope | SDK release (crates.io) |
| `PYPI_TOKEN` | PyPI API token scoped to the `hook0-client` project. Passed as the password with the user name `__token__`, which is what PyPI expects. | SDK release (PyPI) |
| `RUBYGEMS_API_KEY` | RubyGems API key with the `push_rubygem` scope. The gemspec sets `rubygems_mfa_required`, so the key has to be one `gem push` can use without prompting for an OTP — otherwise the job hangs until its timeout. | SDK release (RubyGems) |
| `NUGET_API_KEY` | nuget.org API key scoped to `Hook0.Client` with Push permission. NuGet keys expire (365 days maximum), so this one has a renewal date. | SDK release (NuGet) |
| `DOCKERHUB_USERNAME` | Docker Hub username | Container release |
| `DOCKERHUB_TOKEN` | Docker Hub access token | Container release |
| `GITHUB_USERNAME` | GitHub username for GHCR | Container release |
| `GITHUB_TOKEN` | GitHub PAT with `packages:write` scope | Container + GitHub release |

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

SDK releases are independent from main releases and publish the TypeScript and Rust clients.

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
4. The tag triggers the SDK release pipeline, which checks the tree agrees with the tag and then publishes each package to its registry.

### What the release covers, and how it is decided

Nothing in `ci/` names an SDK. `ci/release-packages` reads the generator's registry
of targets (`hook0_sdkgen::targets::targets()`), walks up from where each target
lands to the one directory its package occupies under `clients/`, and reads the
package's name and version out of whatever manifest sits there. A target added to
the registry is therefore versioned and published by machinery that already
exists; a target whose package cannot be established stops the release naming
itself and the paths it looked at, rather than shipping forever at whatever
version it was born with.

A package that owns a release flow of its own — `ci/release-<name>.gitlab-ci.yml`,
which today is only `clients/mcp` — has a tag of its own per ADR 0004 and is left
alone by the SDK release.

Run it against a working copy to see the inventory:

```bash
cargo run -p release-packages -- list
```

### Registries with no publish job, and why

Three of the packages the tool discovers have no job in
`ci/release-sdk.gitlab-ci.yml`. Each one is missing something a pipeline cannot
supply on its own, and a job that failed on its first real run would be worse
than an absent one.

- **Go — `github.com/hook0/hook0/clients/go`.** Go has no registry: a module is
  published by pushing a tag the proxy can see, and a module in a subdirectory
  needs a tag naming that subdirectory — `clients/go/vX.Y.Z` — which is not the
  `sdk-vX.Y.Z` this pipeline uses. The module path also points at GitHub, which
  is a mirror of this repository rather than the repository. So today
  `go get github.com/hook0/hook0/clients/go@v1.1.0` resolves nothing, and only
  `@latest` works, answering a pseudo-version built from the mirror's default
  branch (`v0.0.0-<date>-<sha>`). For a real version to work, one of two things
  has to change: either the tag pipeline also pushes `clients/go/vX.Y.Z` to the
  GitHub mirror once the mirror has the commit, or the Go client moves to a
  repository of its own and the module path follows it. The first is a few lines;
  what makes it a decision rather than a task is that it puts the release on the
  mirror's lag, which has already broken release-note generation before (see the
  comment at the top of `ci/extract-release-notes.sh`).

- **Packagist — `hook0/client`.** Packagist does not accept uploads and does not
  read subdirectories: a package is a repository, and its versions are that
  repository's tags. `clients/php` is a directory in a monorepo, so it cannot be
  submitted as-is. Publishing it means maintaining a read-only split repository
  (`splitsh-lite` or `git subtree split`), pushing `vX.Y.Z` tags to it, and
  registering that repository once. The split is scriptable; deciding to have a
  second repository is not.

- **Maven Central — `com.hook0:hook0-client`.** Two things are missing, and only
  one of them is a job. Before anything can ever be published, a human has to
  claim the `com.hook0` namespace on the Central Portal and prove control of the
  domain by publishing the TXT record Sonatype asks for; and `clients/java/pom.xml`
  has to gain the plugins a Central release needs — `central-publishing-maven-plugin`,
  `maven-gpg-plugin`, and the source and javadoc jars Central rejects a release
  without. Once both are done the job needs four variables that do not exist yet:
  a base64 GPG private key, its passphrase, and the Central Portal token pair.
  None of them is invented here.

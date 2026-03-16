# CI/CD Configuration

## Required CI Variables for Release Jobs

The manual release jobs (`release:patch`, `release:minor`, `release:major`) and SDK release jobs require the following CI/CD variables:

| Variable | Description | Required For |
|----------|-------------|--------------|
| `GITLAB_RELEASE_TOKEN` | Personal access token with `write_repository` scope | All releases |
| `NPM_TOKEN` | npm automation token for publishing | SDK release |
| `CARGO_TOKEN` | crates.io API token | SDK release |
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
   - Bump versions in clients/rust/Cargo.toml and clients/typescript/package.json
   - Create a commit with the version bump
   - Create and push a git tag (e.g., sdk-v1.0.1)
4. The tag triggers the SDK release pipeline which:
   - Publishes `hook0-client` to npm
   - Publishes `hook0-client` to crates.io

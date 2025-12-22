# CI/CD Configuration

## Required CI Variables for Release Jobs

The manual release jobs (`release:patch`, `release:minor`, `release:major`) require the following CI/CD variable:

### GITLAB_RELEASE_TOKEN (Required)

Personal Access Token with `write_repository` scope.

**Setup in GitLab:**
1. Go to https://gitlab.com/-/profile/personal_access_tokens
2. Create a token with `write_repository` scope
3. Go to Settings > CI/CD > Variables
4. Add variable `GITLAB_RELEASE_TOKEN` with the token value
5. Mark as "Protected" and "Masked"

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
   - Builds binaries for linux/amd64 and linux/arm64
   - Builds frontend
   - Builds and pushes multi-arch Docker images
   - Creates GitLab and GitHub releases with all artifacts

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
    ├─► release.build-api-arm64
    ├─► release.build-worker-amd64
    ├─► release.build-worker-arm64
    ├─► release.build-frontend
    ├─► release.extract-notes
    │
    ├─► release.containers (multi-arch Docker images)
    │       ├─► GitLab Registry
    │       ├─► DockerHub
    │       └─► GHCR
    │
    └─► release.github (GitHub release with binaries)
```

## Artifacts

### Binaries
- `hook0-api-linux-amd64`
- `hook0-api-linux-arm64`
- `hook0-output-worker-linux-amd64`
- `hook0-output-worker-linux-arm64`
- `frontend-dist.tar.gz`

### Docker Images
- `rbaumier/hook0-api:<version>` (DockerHub)
- `rbaumier/output-worker:<version>` (DockerHub)
- `ghcr.io/rbaumier/hook0-api:<version>` (GitHub Container Registry)
- `ghcr.io/rbaumier/output-worker:<version>` (GitHub Container Registry)
- `$CI_REGISTRY_IMAGE/hook0-api:<version>` (GitLab Registry)
- `$CI_REGISTRY_IMAGE/output-worker:<version>` (GitLab Registry)
# Test commit

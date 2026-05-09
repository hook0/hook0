#!/bin/bash
# Per-package release driver for Hook0.
#
# Usage: ./ci/pre-release.sh <package> <patch|minor|major>
#
# Packages: api, output-worker, cli, play, frontend, mcp
# Tag convention: <package>/v<X>.<Y>.<Z> (see adr/0004-monorepo-tag-convention.md)
#
# Pre-release versions (e.g. 1.0.0-alpha.3) are accepted as input. The suffix
# is dropped before bumping, so:
#   1.0.0-alpha.3 + patch -> 1.0.1
#   1.0.0-alpha.3 + minor -> 1.1.0
#   1.0.0-alpha.3 + major -> 2.0.0
set -euo pipefail

PACKAGE="${1:-}"
BUMP_TYPE="${2:-}"

if [ -z "$PACKAGE" ] || [ -z "$BUMP_TYPE" ]; then
    echo "ERROR: Package and bump type required"
    echo "Usage: $0 <package> <patch|minor|major>"
    echo "Packages: api, output-worker, cli, play, frontend, mcp"
    exit 1
fi

for cmd in jq cargo git-cliff; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "ERROR: Required command '$cmd' not found"
        exit 1
    fi
done

# Run from repo root
cd "$(dirname "$0")/.."

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "master" ]; then
    echo "ERROR: Releases must be created from master branch (currently on '$CURRENT_BRANCH')"
    exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
    echo "ERROR: Working directory is not clean. Commit or stash changes first."
    exit 1
fi

# Per-package descriptor
case "$PACKAGE" in
    api)
        CARGO_NAME="hook0-api"
        VERSION_FILE="api/Cargo.toml"
        VERSION_TYPE="cargo"
        CHANGELOG_PATH="api/CHANGELOG.md"
        INCLUDE_PATH="api/**"
        ;;
    output-worker)
        CARGO_NAME="hook0-output-worker"
        VERSION_FILE="output-worker/Cargo.toml"
        VERSION_TYPE="cargo"
        CHANGELOG_PATH="output-worker/CHANGELOG.md"
        INCLUDE_PATH="output-worker/**"
        ;;
    cli)
        CARGO_NAME="hook0-cli"
        VERSION_FILE="cli/Cargo.toml"
        VERSION_TYPE="cargo"
        CHANGELOG_PATH="cli/CHANGELOG.md"
        INCLUDE_PATH="cli/**"
        ;;
    play)
        CARGO_NAME="hook0-play"
        VERSION_FILE="play/Cargo.toml"
        VERSION_TYPE="cargo"
        CHANGELOG_PATH="play/CHANGELOG.md"
        INCLUDE_PATH="play/**"
        ;;
    frontend)
        CARGO_NAME=""
        VERSION_FILE="frontend/package.json"
        VERSION_TYPE="npm"
        CHANGELOG_PATH="frontend/CHANGELOG.md"
        INCLUDE_PATH="frontend/**"
        ;;
    mcp)
        CARGO_NAME="hook0-mcp"
        VERSION_FILE="clients/mcp/Cargo.toml"
        VERSION_TYPE="cargo"
        CHANGELOG_PATH="clients/mcp/CHANGELOG.md"
        INCLUDE_PATH="clients/mcp/**"
        ;;
    *)
        echo "ERROR: Unknown package '$PACKAGE'. Valid: api, output-worker, cli, play, frontend, mcp"
        exit 1
        ;;
esac

# Read current version
case "$VERSION_TYPE" in
    cargo)
        CURRENT=$(grep '^version = ' "$VERSION_FILE" | head -1 | sed 's/version = "\(.*\)"/\1/')
        ;;
    npm)
        CURRENT=$(jq -r '.version' "$VERSION_FILE")
        ;;
esac

if [ -z "$CURRENT" ] || [ "$CURRENT" = "null" ]; then
    echo "ERROR: Could not read version from $VERSION_FILE"
    exit 1
fi

# Accept either stable (X.Y.Z) or pre-release (X.Y.Z-suffix)
if ! echo "$CURRENT" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
    echo "ERROR: Invalid version format '$CURRENT' in $VERSION_FILE"
    exit 1
fi

# Drop any pre-release suffix before computing the bump target
BASE=$(echo "$CURRENT" | sed 's/-.*//')

case "$BUMP_TYPE" in
    patch) NEW_VERSION=$(echo "$BASE" | awk -F. '{print $1"."$2"."$3+1}') ;;
    minor) NEW_VERSION=$(echo "$BASE" | awk -F. '{print $1"."$2+1".0"}') ;;
    major) NEW_VERSION=$(echo "$BASE" | awk -F. '{print $1+1".0.0"}') ;;
    *)
        echo "ERROR: Invalid bump type '$BUMP_TYPE'. Use patch, minor, or major."
        exit 1
        ;;
esac

TAG_PREFIX="${PACKAGE}/v"
TAG="${TAG_PREFIX}${NEW_VERSION}"

echo "=== Releasing $PACKAGE: $CURRENT -> $NEW_VERSION (tag: $TAG) ==="

# Apply version bump
case "$VERSION_TYPE" in
    cargo)
        cargo release version "$NEW_VERSION" --execute --no-confirm --package "$CARGO_NAME"
        cargo update --workspace
        ;;
    npm)
        jq ".version = \"${NEW_VERSION}\"" "$VERSION_FILE" > "$VERSION_FILE.tmp"
        mv "$VERSION_FILE.tmp" "$VERSION_FILE"
        ;;
esac

# Per-package CHANGELOG, scoped by path and by tag prefix
mkdir -p "$(dirname "$CHANGELOG_PATH")"
if [ ! -f "$CHANGELOG_PATH" ]; then
    cat > "$CHANGELOG_PATH" <<EOF
# Changelog — ${PACKAGE}

All notable changes to this package are documented here.
Tags use the convention \`${PACKAGE}/vX.Y.Z\` (see [ADR 0004](../adr/0004-monorepo-tag-convention.md)).
EOF
fi

git-cliff \
    --include-path "$INCLUDE_PATH" \
    --tag-pattern "^${TAG_PREFIX}[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$" \
    --unreleased \
    --prepend "$CHANGELOG_PATH" \
    --tag "$TAG"

# Stage and commit
case "$VERSION_TYPE" in
    cargo)
        git add Cargo.lock "$VERSION_FILE" "$CHANGELOG_PATH"
        ;;
    npm)
        git add "$VERSION_FILE" "$CHANGELOG_PATH"
        ;;
esac

git commit -m "chore(release): bump ${PACKAGE} to ${NEW_VERSION}"
git tag -a "$TAG" -m "Release $TAG"
git push origin HEAD "$TAG"

echo "=== Release $TAG completed ==="

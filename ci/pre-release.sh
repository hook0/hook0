#!/bin/bash
# Release script for Hook0
# Usage: ./ci/pre-release.sh <patch|minor|major>
set -euo pipefail

# Check required tools
for cmd in jq cargo git-cliff; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "ERROR: Required command '$cmd' not found"
        exit 1
    fi
done

BUMP_TYPE="${1:-}"

if [ -z "$BUMP_TYPE" ]; then
    echo "ERROR: Bump type required"
    echo "Usage: $0 <patch|minor|major>"
    exit 1
fi

# Change to repo root (script is in ci/ folder)
cd "$(dirname "$0")/.."

# Safety checks
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "master" ]; then
    echo "ERROR: Releases must be created from master branch (currently on '$CURRENT_BRANCH')"
    exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
    echo "ERROR: Working directory is not clean. Commit or stash changes first."
    exit 1
fi

# Get current version from api/Cargo.toml
CURRENT=$(grep '^version = ' api/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

# Validate version format (must be X.Y.Z with numeric components)
if [ -z "$CURRENT" ]; then
    echo "ERROR: Could not extract version from api/Cargo.toml"
    exit 1
fi

if ! echo "$CURRENT" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "ERROR: Invalid version format '$CURRENT'. Expected semver format (e.g., 1.2.3)"
    exit 1
fi

# Calculate new version
case "$BUMP_TYPE" in
    patch)
        NEW_VERSION=$(echo "$CURRENT" | awk -F. '{print $1"."$2"."$3+1}')
        ;;
    minor)
        NEW_VERSION=$(echo "$CURRENT" | awk -F. '{print $1"."$2+1".0"}')
        ;;
    major)
        NEW_VERSION=$(echo "$CURRENT" | awk -F. '{print $1+1".0.0"}')
        ;;
    *)
        echo "ERROR: Invalid bump type '$BUMP_TYPE'. Use patch, minor, or major."
        exit 1
        ;;
esac

echo "=== Starting $BUMP_TYPE release: $CURRENT -> $NEW_VERSION ==="

# Bump Cargo.toml versions to exact version (no commit, no tag, no push)
echo "Bumping Cargo.toml versions to $NEW_VERSION..."
cargo release version "$NEW_VERSION" --execute --no-confirm

# Regenerate Cargo.lock after version bump
echo "Updating Cargo.lock..."
cargo update --workspace
echo "  ✓ Cargo.lock updated"

# Update frontend/package.json version
echo "Updating frontend/package.json..."
jq ".version = \"${NEW_VERSION}\"" frontend/package.json > frontend/package.json.tmp && mv frontend/package.json.tmp frontend/package.json
echo "  ✓ frontend/package.json updated"

# Generate changelog with git-cliff
echo "Generating CHANGELOG.md..."
git-cliff -o CHANGELOG.md --tag "v${NEW_VERSION}"
echo "  ✓ CHANGELOG.md generated"

# Commit all changes, tag, and push
git add Cargo.lock api/Cargo.toml output-worker/Cargo.toml frontend/package.json CHANGELOG.md
git commit -m "chore(release): bump version to ${NEW_VERSION}"
git tag -a "v${NEW_VERSION}" -m "Release ${NEW_VERSION}"
git push origin HEAD "v${NEW_VERSION}"

echo "=== Release $NEW_VERSION completed ==="

#!/bin/bash
# Release script for Hook0
# Usage: ./ci/pre-release.sh <patch|minor|major>
set -e

BUMP_TYPE="$1"

if [ -z "$BUMP_TYPE" ]; then
    echo "ERROR: Bump type required"
    echo "Usage: $0 <patch|minor|major>"
    exit 1
fi

# Change to repo root (script is in ci/ folder)
cd "$(dirname "$0")/.."

# Get current version from api/Cargo.toml
CURRENT=$(grep '^version = ' api/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

# Calculate new version
case "$BUMP_TYPE" in
    patch)
        NEW_VERSION=$(echo $CURRENT | awk -F. '{print $1"."$2"."$3+1}')
        ;;
    minor)
        NEW_VERSION=$(echo $CURRENT | awk -F. '{print $1"."$2+1".0"}')
        ;;
    major)
        NEW_VERSION=$(echo $CURRENT | awk -F. '{print $1+1".0.0"}')
        ;;
    *)
        echo "ERROR: Invalid bump type '$BUMP_TYPE'. Use patch, minor, or major."
        exit 1
        ;;
esac

echo "=== Starting $BUMP_TYPE release: $CURRENT -> $NEW_VERSION ==="

# Bump Cargo.toml versions only (no commit, no tag, no push)
echo "Bumping Cargo.toml versions..."
cargo release version "$BUMP_TYPE" --execute --no-confirm

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

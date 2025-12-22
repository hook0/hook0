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

# Update frontend/package.json version
echo "Updating frontend/package.json..."
jq ".version = \"${NEW_VERSION}\"" frontend/package.json > frontend/package.json.tmp && mv frontend/package.json.tmp frontend/package.json
echo "  ✓ frontend/package.json updated"

# Generate changelog with git-cliff
echo "Generating CHANGELOG.md..."
git-cliff -o CHANGELOG.md --tag "v${NEW_VERSION}"
echo "  ✓ CHANGELOG.md generated"

# Stage the changed files so they're included in cargo-release commit
git add frontend/package.json CHANGELOG.md

# Bump Cargo.toml versions, commit, tag, and push
# --allow-dirty is needed because we staged frontend/package.json and CHANGELOG.md above
echo "Running cargo release..."
cargo release "$BUMP_TYPE" --execute --no-confirm --allow-dirty

echo "=== Release $NEW_VERSION completed ==="

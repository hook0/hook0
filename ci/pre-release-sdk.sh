#!/bin/bash
# SDK Release script for Hook0
# Usage: ./ci/pre-release-sdk.sh <patch|minor|major>
set -euo pipefail

# Check required tools
for cmd in jq git sed awk cargo; do
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
# TODO: revert to master-only before merge
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "master" ]; then
    echo "WARNING: Releasing from non-master branch '$CURRENT_BRANCH' (testing mode)"
fi

if [ -n "$(git status --porcelain)" ]; then
    echo "ERROR: Working directory is not clean. Commit or stash changes first."
    exit 1
fi

# Get current version from clients/rust/Cargo.toml
CURRENT=$(grep '^version = ' clients/rust/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

# Validate version was extracted
if [ -z "$CURRENT" ] || ! [[ "$CURRENT" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "ERROR: Could not extract valid semver from clients/rust/Cargo.toml"
    echo "Found: '$CURRENT'"
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

echo "=== Starting SDK $BUMP_TYPE release: $CURRENT -> $NEW_VERSION ==="

# Update clients/rust/Cargo.toml
echo "Updating clients/rust/Cargo.toml..."
sed -i.bak "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" clients/rust/Cargo.toml
rm -f clients/rust/Cargo.toml.bak
echo "  ✓ clients/rust/Cargo.toml updated"

# Update clients/typescript/package.json
echo "Updating clients/typescript/package.json..."
jq ".version = \"${NEW_VERSION}\"" clients/typescript/package.json > clients/typescript/package.json.tmp && mv clients/typescript/package.json.tmp clients/typescript/package.json
echo "  ✓ clients/typescript/package.json updated"

# Update api/Cargo.toml hook0-client dependency version
echo "Updating api/Cargo.toml hook0-client dependency..."
sed -i.bak "s/\(hook0-client.*version = \"\)$CURRENT/\1$NEW_VERSION/" api/Cargo.toml
rm -f api/Cargo.toml.bak
echo "  ✓ api/Cargo.toml updated"

# Regenerate Cargo.lock
echo "Updating Cargo.lock..."
cargo update -p hook0-client
echo "  ✓ Cargo.lock updated"

# Commit all changes, tag, and push
git add clients/rust/Cargo.toml clients/typescript/package.json api/Cargo.toml Cargo.lock
git commit -m "chore(release): bump SDK version to ${NEW_VERSION}"
git tag -a "sdk-v${NEW_VERSION}" -m "SDK Release ${NEW_VERSION}"
git push origin HEAD "sdk-v${NEW_VERSION}"

echo "=== SDK Release $NEW_VERSION completed ==="

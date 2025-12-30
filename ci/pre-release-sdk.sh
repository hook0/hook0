#!/bin/bash
# SDK Release script for Hook0
# Usage: ./ci/pre-release-sdk.sh <patch|minor|major>
set -euo pipefail

BUMP_TYPE="$1"

if [ -z "$BUMP_TYPE" ]; then
    echo "ERROR: Bump type required"
    echo "Usage: $0 <patch|minor|major>"
    exit 1
fi

# Change to repo root (script is in ci/ folder)
cd "$(dirname "$0")/.."

# Get current version from clients/rust/Cargo.toml
CURRENT=$(grep '^version = ' clients/rust/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

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

# Commit all changes, tag, and push
git add clients/rust/Cargo.toml clients/typescript/package.json
git commit -m "chore(release): bump SDK version to ${NEW_VERSION}"
git tag -a "sdk-v${NEW_VERSION}" -m "SDK Release ${NEW_VERSION}"
git push origin HEAD "sdk-v${NEW_VERSION}"

echo "=== SDK Release $NEW_VERSION completed ==="

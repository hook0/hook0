#!/bin/sh
# Generate per-package release notes for a tag pipeline, with a fail-safe fallback.
# POSIX shell (busybox ash on the alpine release-notes job — alpine has no /bin/bash).
#
# Usage: ./ci/extract-release-notes.sh <include-path> <tag-pattern> <output-file>
#
# git-cliff auto-detects GitHub from any remote that points at github.com and
# queries the GitHub commits API. When the github.com mirror is behind GitLab
# master, git-cliff hits 404 and panics (observed on play/v1.0.1 and on the
# orphan output-worker bump commit). The fallback emits a plain `git log`
# extract so the rest of the release flow can proceed.
set -euo pipefail

INCLUDE_PATH="${1:?include path required, e.g. 'api/**'}"
TAG_PATTERN="${2:?tag pattern required, e.g. '^api/v[0-9]+\\.[0-9]+\\.[0-9]+(-[a-zA-Z0-9.]+)?\$'}"
OUTPUT="${3:?output file required}"

mkdir -p "$(dirname "$OUTPUT")"

CLIFF_LOG=$(mktemp)
trap 'rm -f "$CLIFF_LOG"' EXIT

if git-cliff \
    --include-path "$INCLUDE_PATH" \
    --tag-pattern "$TAG_PATTERN" \
    --current \
    --strip header \
    > "$OUTPUT" 2>"$CLIFF_LOG"; then
    echo "git-cliff produced $(wc -l <"$OUTPUT") lines for ${CI_COMMIT_TAG:-(no tag)}"
    exit 0
fi

echo "git-cliff failed for ${CI_COMMIT_TAG:-(no tag)} — log:"
cat "$CLIFF_LOG"
echo
echo "Falling back to git log for $INCLUDE_PATH"

PKG_DIR="${INCLUDE_PATH%/**}"
{
    echo "## ${CI_COMMIT_TAG:-Release}"
    echo
    git log --pretty='format:- %s' --no-merges -30 -- "$PKG_DIR/"
    echo
} > "$OUTPUT"

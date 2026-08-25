#!/bin/bash
# SDK release script for Hook0.
#
# Usage: ./ci/pre-release-sdk.sh <patch|minor|major>
#
# Every client that does not own a release flow goes out together under one tag,
# `sdk-vX.Y.Z` (grandfathered by adr/0004-monorepo-tag-convention.md).
#
# Which packages that is, where each of them keeps its version, and what each is
# published as are read out of the tree by ci/release-packages. Nothing is named
# here on purpose: this script used to bump three files by name, so a client that
# was not one of those three shipped at whatever version it was born with and
# nothing ever went red about it.
set -euo pipefail

BUMP_TYPE="${1:-}"

if [ -z "$BUMP_TYPE" ]; then
    echo "ERROR: Bump type required"
    echo "Usage: $0 <patch|minor|major>"
    exit 1
fi

for cmd in git awk sed cargo git-cliff; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "ERROR: Required command '$cmd' not found"
        exit 1
    fi
done

# Run from repo root (script is in ci/)
cd "$(dirname "$0")/.."

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "master" ]; then
    echo "ERROR: SDK releases must be created from master branch (currently on '$CURRENT_BRANCH')"
    exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
    echo "ERROR: Working directory is not clean. Commit or stash changes first."
    exit 1
fi

PACKAGES="cargo run --quiet --locked -p release-packages --"

echo "=== What this release covers ==="
$PACKAGES list

# The version to bump from is the one every SDK is at. Two of them disagreeing is
# a release that already went wrong, and the tool refuses rather than picking one.
CURRENT=$($PACKAGES current)

# The directory of every package this tag covers. It scopes each changelog below,
# and it scopes the commits the guard reads: a commit touching none of them says
# nothing about this release.
DIRECTORIES=$($PACKAGES directories)

# Before anything is written: hold the release to the commits it is made of.
#
# The bump is an argument, and the version below is computed by adding one to the
# last release — so nothing here knows what happened since, and a change that
# breaks a client goes out as a patch the moment somebody asks for one. What
# happened since is in the commit messages, and release-packages reads them: a
# bump smaller than they demand is refused naming the commits that demand more.
# A larger one is left alone, since deciding a release is a major is a decision
# no commit log can overrule.
#
# Unquoted on purpose: one path per package, which is one argument per package.
# shellcheck disable=SC2086
$PACKAGES required-bump "$BUMP_TYPE" 'sdk-v*' $DIRECTORIES

case "$BUMP_TYPE" in
    patch) NEW_VERSION=$(echo "$CURRENT" | awk -F. '{print $1"."$2"."$3+1}') ;;
    minor) NEW_VERSION=$(echo "$CURRENT" | awk -F. '{print $1"."$2+1".0"}') ;;
    major) NEW_VERSION=$(echo "$CURRENT" | awk -F. '{print $1+1".0.0"}') ;;
    *)
        echo "ERROR: Invalid bump type '$BUMP_TYPE'. Use patch, minor, or major."
        exit 1
        ;;
esac

TAG="sdk-v${NEW_VERSION}"

echo "=== Starting SDK $BUMP_TYPE release: $CURRENT -> $NEW_VERSION (tag: $TAG) ==="

# Every package that declares a version, written and read back. A package the
# host versions by its tag — a Go module, a Composer package — says so and is
# left alone.
$PACKAGES set-version "$NEW_VERSION"

# The API depends on the Rust client by version, so it follows the release rather
# than resolving to a crate that is not published yet.
echo "Updating api/Cargo.toml hook0-client dependency..."
sed -i.bak "s/\(hook0-client.*version = \"\)$CURRENT/\1$NEW_VERSION/" api/Cargo.toml
rm -f api/Cargo.toml.bak
# Every committed lockfile, not only the one at the root. A client's version is written into the
# lock of any workspace that depends on it by path, and those workspaces are not members of the
# root one, so a single `cargo update --workspace` here leaves them behind: after sdk-v2.0.0 was
# tagged, `smoke/languages/rust/Cargo.lock` still said 1.1.0 and `cargo tree --locked` refused it.
#
# Found rather than listed, the same way smoke/tests/lockfiles.rs finds them, so a workspace added
# later is re-resolved without editing this.
LOCKFILES=$(git ls-files '*Cargo.lock')
while IFS= read -r lock; do
    [ -n "$lock" ] || continue
    cargo update --workspace --manifest-path "$(dirname "$lock")/Cargo.toml"
done <<< "$LOCKFILES"

# One changelog per package, scoped to the package's own directory and to the
# tags this release uses, as ADR 0004 requires. The set of packages is the one
# read above rather than a list written here.
#
# Each file is written whole rather than prepended to. `--prepend` inserts the
# document it just generated at byte zero, header and all, so a file that has
# been through two releases carries two `# Changelog` headings and a third is on
# the way — which is what api/CHANGELOG.md and clients/mcp/CHANGELOG.md look like
# today. Regenerating is idempotent instead: the same commits produce the same
# file, and the pending ones move from `[Unreleased]` into the section the tag
# names.
while IFS= read -r directory; do
    [ -n "$directory" ] || continue

    git-cliff \
        --include-path "${directory}/**" \
        --tag-pattern '^sdk-v[0-9]+\.[0-9]+\.[0-9]+$' \
        --tag "$TAG" \
        --output "${directory}/CHANGELOG.md"

    # Five clients also carry the version as a constant in their own source, because that is what
    # goes in the `User-Agent` they introduce themselves with. `set-version` writes manifests and a
    # source file is not one, so a bump left them announcing the release before: sdk-v2.0.0 built as
    # 2.0.0 and said `hook0-client-typescript/1.1.0` on the wire, which the conformance corpus
    # caught only once the tag had been pushed.
    #
    # Which files to look in is asked of git rather than listed, so a client that grows a second one
    # is covered without editing this. Only a line whose name reads as `version` is rewritten, which
    # leaves a dependency pinned at the same number where it is. The guard in release-packages holds
    # the tree to exactly this rule.
    git ls-files -z -- "$directory" | while IFS= read -r -d '' file; do
        case "$file" in
            *.md | *[Ll]ock* | *lock*) continue ;;
        esac
        sed -i -E \
            "s/([Vv][Ee][Rr][Ss][Ii][Oo][Nn]_*[[:space:]]*=[[:space:]]*['\"])${CURRENT}(['\"])/\\1${NEW_VERSION}\\2/g" \
            "$file"
    done

    # A package that fetches its own source names the release it is part of, and that name is a tag
    # rather than a version. `set-version` writes manifests, and the tag inside a rockspec is not
    # one, so a bump wrote `2.0.0` beside `tag = "sdk-v1.1.0"`: a rock announcing one release and
    # installing the one before it, which nothing downstream of the install could tell apart.
    #
    # The tag of the release before is the tag of this one with the old version in it, so neither is
    # spelled out twice. This runs before the rename below, because after a `mv` git still lists the
    # path the file no longer has.
    escaped_current=$(printf %s "$CURRENT" | sed 's/\./\\./g')
    previous_tag="${TAG%$NEW_VERSION}${escaped_current}"
    git ls-files -z -- "$directory" | while IFS= read -r -d '' file; do
        case "$file" in
            *.md | *[Ll]ock* | *lock*) continue ;;
        esac
        sed -i -E "s/${previous_tag}/${TAG}/g" "$file"
    done

    # A file whose name carries the version, which is what LuaRocks asks for: a rockspec is named
    # `<package>-<version>-<revision>.rockspec`, and `spec/rockspec_spec.lua` refuses one declaring
    # a version it is not named after. `set-version` writes contents, and a file name is not one, so
    # a bump left the rockspec declaring 2.0.0 under its 1.1.0 name and the Lua suite stopped the
    # release on it.
    #
    # Asked of git and matched on the version rather than on an extension, so a client that starts
    # naming a file after the release is carried without editing this.
    git ls-files -z -- "$directory" | while IFS= read -r -d '' file; do
        base=$(basename "$file")
        case "$base" in
            *"$CURRENT"*) ;;
            *) continue ;;
        esac
        mv "$file" "$(dirname "$file")/$(printf %s "$base" | sed "s/${escaped_current}/${NEW_VERSION}/g")"
    done

    # The four SDKs no registry resolves tell a reader to build from a checkout or to fetch a tag,
    # and so they name a version in prose. `set-version` writes manifests, and a README is not one,
    # so those four went on advertising the release before last until somebody noticed. The shapes
    # below are the ones a version appears in there, and the guard in release-packages holds the
    # tree to them: a version written some other way is caught rather than quietly left behind.
    [ -f "${directory}/README.md" ] || continue
    sed -i \
        -e "s|<version>${CURRENT}</version>|<version>${NEW_VERSION}</version>|g" \
        -e "s|:${CURRENT}\([^0-9]\)|:${NEW_VERSION}\1|g" \
        -e "s|-${CURRENT}-|-${NEW_VERSION}-|g" \
        -e "s|/v${CURRENT}\.|/v${NEW_VERSION}.|g" \
        "${directory}/README.md"

done <<< "$DIRECTORIES"

# Everything the two steps above touched, which is what a clean working directory
# at the top of this script makes safe to stage without naming a single file.
git add -A -- clients api/Cargo.toml
while IFS= read -r lock; do
    [ -n "$lock" ] || continue
    git add -A -- "$lock"
done <<< "$LOCKFILES"
git commit -m "chore(release): bump SDK version to ${NEW_VERSION}"
git tag -a "$TAG" -m "SDK Release ${NEW_VERSION}"
git push origin HEAD "$TAG"

echo "=== SDK Release $NEW_VERSION completed ==="

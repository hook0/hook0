#!/usr/bin/env bash
#
# What kompose would make of the compose file, held against what is committed here.
#
# These manifests were first bootstrapped by running kompose over `docker-compose.yaml`, and they
# have been maintained by hand ever since. The chart in particular: its API deployment reads the
# Biscuit key and the database URL out of a Secret, its images name a registry, its replica counts
# are not one, and its Chart.yaml carries a name, maintainers and an icon. None of that is anything
# kompose emits. Re-running it over these directories put every one of those back to the compose
# file's development value in plain text — the private key included — and overwrote the chart's
# README with a kompose stub. So this writes nothing here any more: it converts into a directory of
# its own and reports, and applying any part of what it reports is a decision a person makes.
#
# Differences are expected, and most of them are the point: a compose file describes a development
# stack and these manifests describe a cluster, so they are not two copies of one thing. What must
# not appear here is a setting the compose file marks as belonging to the development stack only.
# That is not left to this script being run — `ci/release-packages` holds it as a test.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
COMPOSE="$REPO/docker-compose.yaml"

if ! command -v kompose > /dev/null; then
    echo "kompose is not on the PATH, and it is what this reads the compose file with." >&2
    exit 1
fi

OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

# Kept rather than dropped, and shown only when a conversion fails: kompose says something about
# every service on every run, and a report nobody can read is one nobody reads.
convert() {
    if ! kompose --provider=kubernetes -f "$COMPOSE" convert \
        --with-kompose-annotation=false --namespace=hook0 "$@" > "$OUT/kompose.log" 2>&1; then
        cat "$OUT/kompose.log" >&2
        exit 1
    fi
}
convert -o "$OUT/kubernetes/deployments.yaml"
(cd "$OUT" && mkdir -p helm && convert --chart --out helm)

echo "== settings the compose file marks as belonging to the development stack only"
echo "   (none of these may appear below, and none of them appear in what is committed)"
awk '
    /^ *#/ { if (tolower($0) ~ /development stack only/) marked = 1; next }
    /^ *- [A-Z][A-Z0-9_]*=/ { if (marked) { sub(/^ *- /, ""); sub(/=.*/, ""); print "   " $0 } }
    { marked = 0 }
' "$COMPOSE"

echo
echo "== what a regeneration would differ by (nothing here has been written to)"
committed_files="$(cd "$HERE" && find kubernetes helm -type f | sort)"
produced_files="$(cd "$OUT" && find kubernetes helm -type f | sort)"
for file in $(printf '%s\n%s\n' "$committed_files" "$produced_files" | sort -u); do
    if [ ! -e "$HERE/$file" ]; then
        echo "-- $file: a regeneration would add this file"
    elif [ ! -e "$OUT/$file" ]; then
        echo "-- $file: kept here, and a regeneration produces no such file"
    else
        diff -u "$HERE/$file" "$OUT/$file" > /dev/null && continue
        echo "-- $file"
        # `|| true` because a difference is what this is looking for, and `diff` says so by
        # exiting non-zero. Without it the report stopped after its first entry, under `pipefail`,
        # and looked exactly like a report with one entry in it.
        { diff -u "$HERE/$file" "$OUT/$file" || true; } | tail -n +3
    fi
done

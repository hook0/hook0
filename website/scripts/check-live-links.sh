#!/usr/bin/env bash
# Live-site link check: curl only, no LLM. Run it every SEO cycle:
#   website/scripts/check-live-links.sh [https://www.hook0.com] [extra-urls.txt]
#
# Design: read what crawlers read first (sitemap, robots.txt, llms.txt), then the
# canonical, hreflang and links of every sitemap page, then the paths people
# guess (/faq, /contact...). Every distinct URL is requested once.
#
# Why throttled: one request per second at most, so the check never looks like a
# crawl against our own CDN. Redirects are followed (max 5); only the final
# status counts, so a 301 to a live page passes. A junk path answering 200 is
# reported as a soft 404.
#
# The optional second argument is a file with one URL per line (the 404 list
# exported from Search Console). Keep that file out of the repo: it is business
# data.
#
# Output ends with "N URLs checked, X in 404 ..." for the cycle trace; the
# operator adds "Y fixed" after correcting. Exit code 1 when X > 0.
set -u
BASE="${1:-https://www.hook0.com}"
EXTRA="${2:-}"
HOST="${BASE#*://}"
W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
: > "$W/seen"; : > "$W/bad"; : > "$W/ext"; : > "$W/blocked"

# check <url> <why>: requests the URL once (deduped through $W/seen); records
# anything that does not end in 2xx. Counters live in files because the
# callers run inside pipelines (subshells).
check() {
  grep -qxF "$1" "$W/seen" && return
  echo "$1" >> "$W/seen"
  sleep 1
  local code
  code="$(curl -s -o /dev/null -L --max-redirs 5 --max-time 20 -w '%{http_code}' -A 'hook0-link-check/1' "$1")" || code=000
  case "$code" in
    2*) ;;
    # 403/429 from a third-party host is bot protection, not a dead link: listed
    # apart so a human can open it, never counted as 404.
    403|429) case "$1" in *"$HOST"*) echo "$code  $1  <- $2" | tee -a "$W/bad";; *) echo "$code  $1  <- $2" >> "$W/blocked";; esac;;
    *) echo "$code  $1  <- $2" | tee -a "$W/bad";;
  esac
}

# abs <page-url> <href>: absolute http(s) URL, empty for mailto:, #, javascript:.
abs() {
  case "$2" in
    http*) echo "$2";; //*) echo "https:$2";; /*) echo "$BASE$2";;
    mailto:*|tel:*|javascript:*|\#*|"") ;; *) echo "${1%/*}/$2";;
  esac
}

fetch() { sleep 1; curl -s -L --max-time 20 -A 'hook0-link-check/1' "$1"; }

echo "== sitemap"
fetch "$BASE/sitemap.xml" | grep -o '<loc>[^<]*</loc>' | sed 's/<[^>]*>//g' > "$W/pages"
[ -s "$W/pages" ] || { echo "sitemap empty or unreachable"; exit 2; }
while read -r u; do check "$u" "sitemap"; done < "$W/pages"

echo "== robots.txt, llms.txt"
check "$BASE/robots.txt" "robots"
fetch "$BASE/robots.txt" | grep -i '^Sitemap:' | awk '{print $2}' | tr -d '\r' | while read -r u; do check "$u" "robots Sitemap"; done
fetch "$BASE/llms.txt" | grep -o 'https\?://[^) >"]*' | sed 's/[.,]$//' | sort -u | while read -r u; do check "$u" "llms.txt"; done

echo "== canonical, hreflang and links of every sitemap page"
while read -r p; do
  fetch "$p" > "$W/page.html"
  grep -o '<link[^>]*>' "$W/page.html" | grep -E 'rel="(canonical|alternate)"' | grep -o 'href="[^"]*"' | cut -d'"' -f2 | while read -r h; do check "$h" "canonical/hreflang of $p"; done
  grep -o '<a [^>]*href="[^"]*"' "$W/page.html" | grep -o 'href="[^"]*"' | cut -d'"' -f2 | sed 's/#.*//' | sort -u | while read -r h; do
    a="$(abs "$p" "$h")"; [ -z "$a" ] && continue
    # Cloudflare rewrites mailto: links to /cdn-cgi/l/email-protection#<hash>; the
    # path alone answers 404 once the fragment is stripped, so it is not ours.
    case "$a" in */cdn-cgi/*) continue;; esac
    case "$a" in *"$HOST"*) check "$a" "link in $p";; *) echo "$a|$p" >> "$W/ext";; esac
  done
done < "$W/pages"

echo "== external links"
sort -t'|' -k1,1 -u "$W/ext" | while IFS='|' read -r u p; do check "$u" "external link in $p"; done

echo "== guessed paths and soft 404"
# /faq and /pricing must exist. /contact, /about and /blog are pages the site does
# not have: a true 404 there is the right answer, so it is printed but not counted.
for g in /faq /pricing; do check "$BASE$g" "guessed path"; done
for g in /contact /about /blog; do
  sleep 1; c="$(curl -s -o /dev/null -L --max-time 20 -w '%{http_code}' "$BASE$g")"
  echo "info  $BASE$g answers $c (no such page; 404 expected, anything else deserves a look)"
done
junk="$(sleep 1; curl -s -o /dev/null -w '%{http_code}' --max-time 20 "$BASE/this-path-does-not-exist-$$")"
[ "$junk" = "404" ] || echo "SOFT-404 junk path answered $junk" | tee -a "$W/bad"

if [ -n "$EXTRA" ] && [ -f "$EXTRA" ]; then
  echo "== Search Console 404 list"
  while read -r u; do [ -n "$u" ] && check "$u" "Search Console 404"; done < "$EXTRA"
fi

N=$(wc -l < "$W/seen"); X=$(wc -l < "$W/bad")
echo
[ -s "$W/blocked" ] && { echo "Blocked by third-party bot protection (open by hand, not counted):"; cat "$W/blocked"; }
echo "$N URLs checked, $X in 404 (or 5xx, unreachable, soft 404), Y fixed: fill in after correcting"
[ "$X" -eq 0 ]

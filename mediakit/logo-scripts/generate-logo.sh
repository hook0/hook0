#!/usr/bin/env bash
set -euo pipefail

# Facebook:
#
# Link posts: 1200 x 628 px
# Image posts: 1200 x 630 px
# Cover image: 820 х 312 px
# Profile image: 170 х 170 p
#
# Twitter:
#
# Image posts: 1024 x 675 px
# Cover image: 1500 х 500 px
# Profile image: 400 x 400 p
#
# Instagram:
#
# Image posts: 1080 x 1080 px
# Profile image: 110 x 110 px
# Stories: 1080 x 1920 px
#
# YouTube:
#
# Thumbnail image: 1280 x 720 px
# Cover image: 2560 x 1440 px
# Profile image: 800 x 800 px
#
# Pinterest:
#
# Image posts: 1000 x 1500 px
# Pin: 236 px width
# Profile image: 165 x 165 px
#
# LinkedIn:
#
# Link posts: 1200 х 628 px
# Image posts: 1200 x 1200 px
# Cover image: 1536 x 768 px
# Profile image: 300 х 300 px
#
# Tik Tok:
#
# Profile photo: 200 x 200 px
# Video length: 1080 x 1920 px

INKSCAPE="/Applications/Inkscape.app/Contents/MacOS/inkscape"
OUTPUT_DIR="../logo"

# The five hand-authored SVGs in $OUTPUT_DIR (logo.svg, logo-white.svg,
# logo-gray.svg, logo-banner.svg, logo-banner-white.svg) are the source of truth
# this script CONSUMES, not something it produces. Two of them, logo-banner.svg
# and logo-banner-white.svg, are the current brand banner. Bail out before
# touching anything if a tool or a source is missing — the previous version
# deleted the whole directory first and regenerated the banners from a 2023
# raster, so a stale or half-set-up run silently reverted the logo.
command -v convert >/dev/null 2>&1 || { echo "generate-logo: ImageMagick 'convert' not found" >&2; exit 1; }
[ -x "$INKSCAPE" ] || { echo "generate-logo: Inkscape not found at $INKSCAPE" >&2; exit 1; }
for src in raw.svg raw-gray.svg "$OUTPUT_DIR/logo-banner.svg"; do
  [ -f "$src" ] || { echo "generate-logo: missing brand source $src" >&2; exit 1; }
done

mkdir -p "$OUTPUT_DIR"
# Regenerate only the raster and icon outputs. Never the .svg sources above —
# `rm -rf $OUTPUT_DIR` used to erase them with nothing to rebuild them from.
find "$OUTPUT_DIR" -maxdepth 1 -type f \( -name '*.png' -o -name '*.ico' \) -delete
for i in 16 32 48 64 110 170 128 256 312 400 500 512 630 720 820 1024 1080 1500 1920 2048 4096
do
   "$INKSCAPE" --export-type png --export-filename "${OUTPUT_DIR}/${i}x${i}.png" -w $i raw.svg
   "$INKSCAPE" --export-type png --export-filename "${OUTPUT_DIR}/${i}x${i}-gray.png" -w $i raw-gray.svg
   convert -flatten "${OUTPUT_DIR}/${i}x${i}.png" "${OUTPUT_DIR}/${i}x${i}-white.png"
   #convert -negate "${OUTPUT_DIR}/${i}x${i}-white.png" "${OUTPUT_DIR}/${i}x${i}-black.png"
done

for i in 256 312 400 500 512 630 720 820 1024 1080 1500 1920 2048 4096
do
  # Source is the current vector banner logo-banner.svg, not the 2023 raster
  # logo-banner.png that carried the old mark. The SVG is transparent, so it
  # gives the -transparent variant directly; the flat-background variant is that
  # same export composited onto white.
  "$INKSCAPE" --export-type png --export-filename "${OUTPUT_DIR}/${i}x${i}-banner-transparent.png" -w $i "${OUTPUT_DIR}/logo-banner.svg"
  convert "${OUTPUT_DIR}/${i}x${i}-banner-transparent.png" -background white -flatten "${OUTPUT_DIR}/${i}x${i}-banner.png"
done

convert "${OUTPUT_DIR}/16x16.png" "${OUTPUT_DIR}/32x32.png" "${OUTPUT_DIR}/48x48.png" "${OUTPUT_DIR}/favicon.ico"

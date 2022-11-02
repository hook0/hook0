#!/usr/bin/env sh
set -eu

mkdir -p /tmp/linux-gnu/
ldd "$1" | grep so \
| sed -e '/^[^\t]/ d' \
| sed -e 's/\t//' \
| sed -e 's/.*=..//' \
| sed -e 's/ (0.*)//' \
| sed -e 's/linux-vdso.so.1//' \
| grep -v -e '^[[:space:]]*$' \
| sort \
| uniq \
| while read -r in; do
    cp "$in" /tmp/linux-gnu/
done

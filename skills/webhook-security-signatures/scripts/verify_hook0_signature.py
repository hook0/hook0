#!/usr/bin/env python3
"""Verify a Hook0 webhook signature (the `X-Hook0-Signature` header).

Header format:

    t=<unix seconds>,h=<covered header names, space-separated>,v1=<hex HMAC-SHA256>

Older subscriptions may still receive the body-only scheme:

    t=<unix seconds>,v0=<hex HMAC-SHA256>

Signed message (HMAC-SHA256 keyed with the subscription secret):

    v1: "<t>.<names joined by ' '>.<their values, same order, joined by '.'>.<raw body>"
    v0: "<t>.<raw body>"

When both codes are present, v1 is the one verified: a right v0 never rescues a
wrong v1.

Usage:
    python3 verify_hook0_signature.py --selftest
    python3 verify_hook0_signature.py --conformance path/to/signature.json

In your app (Flask example):

    ok, reason = verify(request.get_data(), request.headers,
                        request.headers.get("X-Hook0-Signature", ""), SECRET)
    if not ok:
        abort(401)
"""
import hashlib
import hmac
import json
import sys
import time

DEFAULT_TOLERANCE_SECONDS = 300


class Refused(Exception):
    """A signature that must not be accepted; `code` names why."""

    def __init__(self, code, detail):
        super().__init__(f"{code}: {detail}")
        self.code = code


def _parse(header):
    parts = {}
    for part in header.split(","):
        name, sep, value = part.partition("=")
        if sep:
            parts[name.strip()] = value.strip()
    try:
        timestamp = int(parts["t"])
    except (KeyError, ValueError):
        raise Refused("signature_unreadable", "no readable `t` moment")
    codes = {}
    for scheme in ("v0", "v1"):
        if scheme in parts:
            try:
                code = bytes.fromhex(parts[scheme])
            except ValueError:
                raise Refused("code_not_hexadecimal", f"`{scheme}` is not whole hexadecimal")
            if len(code) != hashlib.sha256().digest_size:
                raise Refused("code_not_hexadecimal", f"`{scheme}` is not a whole SHA-256 code")
            codes[scheme] = code
    if not codes:
        raise Refused("signature_unreadable", "neither a `v0` nor a `v1` code")
    covered = parts.get("h", "").split() if "v1" in codes else []
    return timestamp, covered, codes


def _header(headers, name):
    """Case-insensitive lookup that works for dicts, lists of pairs and framework header objects."""
    items = headers.items() if hasattr(headers, "items") else headers
    for key, value in items:
        if key.lower() == name.lower():
            return value
    return None


def check(body, headers, signature_header, secret, now=None,
          tolerance=DEFAULT_TOLERANCE_SECONDS):
    """Raise `Refused` unless the delivery is authentic and fresh."""
    timestamp, covered, codes = _parse(signature_header)
    now = int(time.time()) if now is None else now
    if abs(now - timestamp) > tolerance:
        raise Refused("outside_tolerance", f"moment {timestamp} is more than {tolerance}s from {now}")

    if "v1" in codes:
        values = []
        for name in covered:
            value = _header(headers, name)
            if value is None:
                raise Refused("header_not_delivered", f"covered header `{name}` was not delivered")
            values.append(value)
        message = f"{timestamp}.{' '.join(covered)}.{'.'.join(values)}.".encode() + body
        expected = codes["v1"]
    else:
        message = f"{timestamp}.".encode() + body
        expected = codes["v0"]

    computed = hmac.new(secret.encode(), message, hashlib.sha256).digest()
    if not hmac.compare_digest(computed, expected):
        raise Refused("code_mismatch", "the code does not match this body and these headers")


def verify(body, headers, signature_header, secret, now=None,
           tolerance=DEFAULT_TOLERANCE_SECONDS):
    """Return (True, "ok") or (False, reason) instead of raising."""
    try:
        check(body, headers, signature_header, secret, now, tolerance)
        return True, "ok"
    except Refused as refused:
        return False, str(refused)


def run_vectors(vectors):
    failures = 0
    for v in vectors:
        try:
            check(v["payload"].encode(), v["headers"], v["signature"], v["secret"],
                  v["current_time"], v["tolerance_seconds"])
            outcome = "accepted"
        except Refused as refused:
            outcome = refused.code
        wanted = "accepted" if v["verdict"] == "accepted" else v["refusal"]
        failures += outcome != wanted
        print(f"{'OK  ' if outcome == wanted else 'FAIL'} {v['name']} (got {outcome}, want {wanted})")
    return failures


# A subset of the vectors every Hook0 SDK is held to (clients/conformance/signature.json
# in the Hook0 repository). Their codes were computed outside any implementation.
_HEADERS = [["x-event-id", "evt-1"], ["x-delivery-id", "dlv-1"], ["content-type", "application/json"]]
SELFTEST_VECTORS = [
    {"name": "v1 header-scheme signature verifies", "verdict": "accepted",
     "signature": "t=1800000000,h=x-event-id x-delivery-id,v1=19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347"},
    {"name": "v0 body-scheme signature verifies", "verdict": "accepted",
     "signature": "t=1800000000,v0=d17d66b66fca89390c5b967c45e8928fc732db07a0aabe8167b1e98213081ffe"},
    {"name": "covered header order is part of what is signed", "verdict": "refused", "refusal": "code_mismatch",
     "signature": "t=1800000000,h=x-delivery-id x-event-id,v1=19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347"},
    {"name": "a covered header that was not delivered", "verdict": "refused", "refusal": "header_not_delivered",
     "signature": "t=1800000000,h=x-event-id x-missing,v1=19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347"},
    {"name": "a code replayed under another moment", "verdict": "refused", "refusal": "code_mismatch",
     "signature": "t=1800000001,v0=d17d66b66fca89390c5b967c45e8928fc732db07a0aabe8167b1e98213081ffe"},
    {"name": "a moment older than the tolerance", "verdict": "refused", "refusal": "outside_tolerance",
     "signature": "t=1799999000,v0=d17d66b66fca89390c5b967c45e8928fc732db07a0aabe8167b1e98213081ffe"},
]
for _v in SELFTEST_VECTORS:
    _v.update(secret="a-subscription-secret", payload='{"event":"user.created"}', headers=_HEADERS,
              current_time=1800000000, tolerance_seconds=300)


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        vectors = SELFTEST_VECTORS
    elif len(sys.argv) == 3 and sys.argv[1] == "--conformance":
        with open(sys.argv[2]) as corpus:
            vectors = json.load(corpus)["vectors"]
    else:
        print(__doc__)
        sys.exit(2)
    failed = run_vectors(vectors)
    print(f"{len(vectors) - failed}/{len(vectors)} vectors behave as expected")
    sys.exit(1 if failed else 0)

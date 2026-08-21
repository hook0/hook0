"""Verifying that a webhook came from Hook0, and that nothing in it changed on the way.

A signature header names the moment it was signed and one or two message authentication codes over
the body. The `v1` scheme also covers a list of request headers, so a receiver can tell apart two
deliveries that carry the same body but not the same context; `v0` covers the body alone and is
what an older sender still produces. When both are offered, `v1` is the one verified: accepting the
weaker of two schemes on the strength of the sender offering it is how a downgrade works.

Two things are refused before any code is computed. A header the signature says it covers but the
request did not carry is refused outright, because signing over an absent value would let a sender
drop a header and keep the signature valid. And a signature whose codes are not whole hexadecimal
is refused rather than decoded as far as it goes: a decoder that stops at the first bad character
compares a prefix, and a prefix of the right code is not the right code.

The clock window is bilateral. A timestamp too far in the future is refused exactly like one too
far in the past, so the window a given delivery is accepted in stays the width the caller asked
for, whichever way a clock drifted.
"""

from __future__ import annotations

import datetime
import hashlib
import hmac
from collections.abc import Iterable, Mapping, Sequence

from .errors import Hook0ClientError

# Longest signature header read. The header is written by whoever reached the endpoint, so its
# size is bounded before any of it is split, decoded or compared.
MAX_SIGNATURE_BYTES = 8 * 1024

# Most `key=value` parts one signature header is split into.
MAX_SIGNATURE_PARTS = 32

# Most header names one signature covers.
MAX_COVERED_HEADERS = 64

# What separates one part of the signature header from the next.
PART_SEPARATOR = ","

# What separates the name of a part from its value. Only the first one counts: a value may hold
# further ones, and splitting on all of them would silently drop everything past the second.
PART_ASSIGNATOR = "="

# What separates two header names inside the `h` part, and what they are joined back with.
HEADER_NAME_SEPARATOR = " "

# What separates the pieces of the message a code is computed over.
MESSAGE_SEPARATOR = "."

# Part naming the moment the delivery was signed, in whole seconds since the Unix epoch.
TIMESTAMP_PART = "t"

# Part carrying the code covering the body alone.
BODY_SCHEME_PART = "v0"

# Part carrying the code covering the covered headers and the body.
HEADERS_SCHEME_PART = "v1"

# Part listing the headers the `v1` code covers, in the order it covers them.
COVERED_HEADERS_PART = "h"

# The characters a hexadecimal code is written with.
HEX_DIGITS = frozenset("0123456789abcdefABCDEF")

# The characters a header name is written with, as RFC 9110 spells a token.
HEADER_NAME_CHARACTERS = frozenset("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!#$%&'*+-.^_`|~")

# What the codes are computed with.
DIGEST = hashlib.sha256

# Furthest from the epoch, in either direction, a signature's moment may sit. Python's integers
# grow without bound, so a header carrying thousands of digits would otherwise reach the arithmetic
# that holds it against the current time and fail there, in a way no caller expects.
MAX_TIMESTAMP = 10**12


class Signature:
    """A signature header, read into the pieces a verification needs."""

    def __init__(
        self,
        timestamp: int,
        covered_headers: Sequence[str],
        body_code: bytes | None,
        headers_code: bytes | None,
    ) -> None:
        self.timestamp = timestamp
        self.covered_headers = list(covered_headers)
        self.body_code = body_code
        self.headers_code = headers_code

    @classmethod
    def parse(cls, signature: str) -> Signature:
        """Reads a signature header, refusing anything it cannot read whole."""
        if not isinstance(signature, str):
            raise Hook0ClientError(f"the signature is {type(signature).__name__}, not a header value")
        if len(signature) > MAX_SIGNATURE_BYTES:
            raise Hook0ClientError(
                f"the signature is {len(signature)} characters long, above the {MAX_SIGNATURE_BYTES} accepted"
            )

        parts = signature.split(PART_SEPARATOR)
        if len(parts) > MAX_SIGNATURE_PARTS:
            raise Hook0ClientError(f"the signature carries more than the {MAX_SIGNATURE_PARTS} parts accepted")

        read: dict[str, str] = {}
        for part in parts:
            named = part.split(PART_ASSIGNATOR, 1)
            if len(named) != 2:
                continue
            read[named[0].strip()] = named[1].strip()

        if len(read) < 2:
            raise Hook0ClientError("the signature carries neither a timestamp nor a code")

        timestamp = _timestamp(read)
        covered_headers = _covered_headers(read)
        body_code = _code(read, BODY_SCHEME_PART)
        headers_code = _code(read, HEADERS_SCHEME_PART)

        if body_code is None and headers_code is None:
            raise Hook0ClientError(
                f"the signature carries neither a `{BODY_SCHEME_PART}` nor a `{HEADERS_SCHEME_PART}` code"
            )

        return cls(timestamp, covered_headers, body_code, headers_code)

    def verify(self, payload: bytes, covered_values: Sequence[str], subscription_secret: str) -> bool:
        """Whether the code this signature carries is the one the secret produces.

        The stronger scheme wins when both are offered, and the comparison is made in constant
        time: one that gave up at the first differing byte would say, by how long it took, how much
        of a guess was right.
        """
        code = hmac.new(subscription_secret.encode("utf-8"), digestmod=DIGEST)
        code.update(str(self.timestamp).encode("utf-8"))
        code.update(MESSAGE_SEPARATOR.encode("utf-8"))

        if self.headers_code is not None:
            code.update(HEADER_NAME_SEPARATOR.join(self.covered_headers).encode("utf-8"))
            code.update(MESSAGE_SEPARATOR.encode("utf-8"))
            code.update(MESSAGE_SEPARATOR.join(covered_values).encode("utf-8"))
            code.update(MESSAGE_SEPARATOR.encode("utf-8"))
            code.update(payload)
            return hmac.compare_digest(code.digest(), self.headers_code)

        if self.body_code is not None:
            code.update(payload)
            return hmac.compare_digest(code.digest(), self.body_code)

        # Unreachable: a signature carrying neither code is refused while it is being read.
        return False


def _timestamp(read: Mapping[str, str]) -> int:
    """The moment the signature names, which it is not a signature without."""
    if TIMESTAMP_PART not in read:
        raise Hook0ClientError(f"the signature carries no `{TIMESTAMP_PART}` part")

    written = read[TIMESTAMP_PART]
    try:
        seconds = int(written)
    except ValueError as unreadable:
        raise Hook0ClientError(f"`{written}` is not a number of seconds") from unreadable

    if abs(seconds) > MAX_TIMESTAMP:
        raise Hook0ClientError(f"the signature's moment is further than {MAX_TIMESTAMP} seconds from the epoch")
    return seconds


def _code(read: Mapping[str, str], part: str) -> bytes | None:
    """One of the codes a signature offers, decoded whole or not at all."""
    if part not in read:
        return None

    written = read[part]
    if len(written) % 2 != 0 or not written or any(digit not in HEX_DIGITS for digit in written):
        raise Hook0ClientError(f"the `{part}` code is not hexadecimal")
    return bytes.fromhex(written)


def _covered_headers(read: Mapping[str, str]) -> list[str]:
    """The headers the stronger scheme covers, in the order it covers them."""
    if COVERED_HEADERS_PART not in read:
        return []

    written = read[COVERED_HEADERS_PART]
    if not written:
        return []

    names = written.split(HEADER_NAME_SEPARATOR)
    if len(names) > MAX_COVERED_HEADERS:
        raise Hook0ClientError(f"the signature covers more than the {MAX_COVERED_HEADERS} headers accepted")

    covered = []
    for name in names:
        if not name or any(character not in HEADER_NAME_CHARACTERS for character in name):
            raise Hook0ClientError(f"`{name}` is not a header name")
        covered.append(name.lower())
    return covered


def _delivered(headers: Mapping[str, str] | Iterable[tuple[str, str]]) -> dict[str, str]:
    """The headers of the request, under the names a signature refers to them by.

    A later value wins over an earlier one under the same name, which is what a mapping would have
    done had the caller built one itself.
    """
    entries: Iterable[tuple[str, str]]
    entries = headers.items() if isinstance(headers, Mapping) else headers

    delivered: dict[str, str] = {}
    for name, value in entries:
        delivered[_text(name).lower()] = _text(value)
    return delivered


def _text(value: str | bytes) -> str:
    """A header name or value as text, whichever way the caller holds it."""
    if isinstance(value, bytes):
        try:
            return value.decode("utf-8")
        except UnicodeDecodeError as unreadable:
            raise Hook0ClientError("a header is not UTF-8") from unreadable
    if isinstance(value, str):
        return value
    raise Hook0ClientError(f"a header is {type(value).__name__}, not a header value")


def verify_webhook_signature_with_current_time(
    signature: str,
    payload: bytes,
    headers: Mapping[str, str] | Iterable[tuple[str, str]],
    subscription_secret: str,
    tolerance: float,
    current_time: datetime.datetime,
) -> bool:
    """Verifies a webhook against a moment the caller names.

    - `signature`: the value of the `X-Hook0-Signature` header.
    - `payload`: the raw body of the webhook request.
    - `headers`: the headers of the webhook request.
    - `subscription_secret`: the signing secret of the subscription the webhook was delivered for.
    - `tolerance`: how far, in seconds and in either direction, the moment the signature names may
      sit from `current_time`. Five minutes is a reasonable trade-off between tolerating clock
      drift and bounding how long a captured delivery can be replayed.
    - `current_time`: what to hold the signature's moment against.

    Returns `True`, and raises `Hook0ClientError` for every reason a webhook may be refused.
    """
    parsed = Signature.parse(signature)

    delivered = _delivered(headers)
    covered_values = []
    for name in parsed.covered_headers:
        if name not in delivered:
            raise Hook0ClientError(f"the `{name}` header the signature covers was not delivered")
        covered_values.append(delivered[name])

    if not parsed.verify(payload, covered_values, subscription_secret):
        raise Hook0ClientError("the signature does not match what the subscription secret produces")

    # A moment carrying no zone is read as UTC rather than as whatever zone the machine running
    # this happens to sit in: the same webhook and the same signature have to be accepted or
    # refused the same way wherever the receiver runs.
    moment = current_time if current_time.tzinfo is not None else current_time.replace(tzinfo=datetime.UTC)

    drift = moment.timestamp() - parsed.timestamp
    if abs(drift) > tolerance:
        raise Hook0ClientError(f"the signature was made {drift:.0f} seconds from now, outside the {tolerance} accepted")

    return True


def verify_webhook_signature(
    signature: str,
    payload: bytes,
    headers: Mapping[str, str] | Iterable[tuple[str, str]],
    subscription_secret: str,
    tolerance: float,
) -> bool:
    """Verifies a webhook against the current moment. See `verify_webhook_signature_with_current_time`."""
    return verify_webhook_signature_with_current_time(
        signature,
        payload,
        headers,
        subscription_secret,
        tolerance,
        datetime.datetime.now(datetime.UTC),
    )

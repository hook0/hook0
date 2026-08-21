"""What the generated half of this package reads and writes values through.

Everything here is hand-written and never regenerated. It is the one seam between what the API
declares — the classes, the problems and the methods the generator writes below this module — and
what it does not: how a JSON document is turned into a typed value, and what happens to a document
that does not say what it was declared to say.

Reading is deliberately strict. A field the document declares as a string and answers as a number
stops the read with the name of the field, rather than yielding an object whose type annotations
lie about what it holds. Every failure of that kind is a `DecodeError`, so a caller has one thing
to catch whatever the shape of the answer was.
"""

from __future__ import annotations

import datetime
import json
import urllib.parse
import uuid
from collections.abc import Callable, Mapping
from enum import StrEnum
from typing import Any, TypeVar

# Longest fragment of a response body an error message carries. Bodies are answered by a server
# this package does not control, so they are cut at a fixed budget rather than echoed whole into
# whatever the caller logs.
MAX_PREVIEW_BYTES = 256

# Largest JSON document read out of a response body, in bytes. The transports cap what they read
# off a socket; this caps what is handed to the parser whichever way the bytes arrived.
MAX_PAYLOAD_BYTES = 8 * 1024 * 1024

Value = TypeVar("Value")

Reader = Callable[[Any], Value]


class DecodeError(ValueError):
    """What the API answered is not what it declares it answers."""


def preview(payload: bytes) -> str:
    """As much of a response body as a message may carry."""
    kept = payload[:MAX_PREVIEW_BYTES]
    rendered = kept.decode("utf-8", errors="replace")
    if len(payload) > MAX_PREVIEW_BYTES:
        return f"{rendered}…"
    return rendered


def decode_payload(payload: bytes) -> Any:
    """The JSON document a response body carries."""
    if len(payload) > MAX_PAYLOAD_BYTES:
        raise DecodeError(f"the response is {len(payload)} bytes, above the {MAX_PAYLOAD_BYTES} accepted")
    try:
        return json.loads(payload)
    except (ValueError, UnicodeDecodeError) as unreadable:
        raise DecodeError(f"the response is not JSON: {preview(payload)}") from unreadable


def as_fields(value: Any, owner: str) -> Mapping[str, Any]:
    """The members of an object the document declares, under the name it declares it with."""
    if not isinstance(value, dict):
        raise DecodeError(f"{owner} is not a JSON object")
    return value


def read(fields: Mapping[str, Any], key: str, reader: Reader[Value]) -> Value:
    """A member the document requires, which is therefore missing when it is absent."""
    if key not in fields:
        raise DecodeError(f"`{key}` is required and was not answered")
    return _named(key, fields[key], reader)


def maybe(fields: Mapping[str, Any], key: str, reader: Reader[Value]) -> Value | None:
    """A member the document does not require, absent as readily as answered as null."""
    if fields.get(key) is None:
        return None
    return _named(key, fields[key], reader)


def _named(key: str, value: Any, reader: Reader[Value]) -> Value:
    """Reads a member, saying which member it was that could not be read."""
    try:
        return reader(value)
    except DecodeError as unreadable:
        raise DecodeError(f"`{key}`: {unreadable}") from unreadable


def as_text(value: Any) -> str:
    """A string, refusing what merely spells like one."""
    if not isinstance(value, str):
        raise DecodeError(f"expected a string, got {type(value).__name__}")
    return value


def as_int(value: Any) -> int:
    """A whole number. `True` is an `int` in Python and is refused here all the same."""
    if isinstance(value, bool) or not isinstance(value, int):
        raise DecodeError(f"expected a whole number, got {type(value).__name__}")
    return value


def as_float(value: Any) -> float:
    """A number, whether the document wrote it with a fractional part or not."""
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise DecodeError(f"expected a number, got {type(value).__name__}")
    return float(value)


def as_bool(value: Any) -> bool:
    """A boolean, refusing the numbers that stand in for one elsewhere."""
    if not isinstance(value, bool):
        raise DecodeError(f"expected a boolean, got {type(value).__name__}")
    return value


def as_uuid(value: Any) -> uuid.UUID:
    """A UUID, as the document spells one."""
    try:
        return uuid.UUID(as_text(value))
    except ValueError as unreadable:
        raise DecodeError(f"expected a UUID, got `{as_text(value)}`") from unreadable


def as_datetime(value: Any) -> datetime.datetime:
    """A moment, as RFC 3339 spells one."""
    try:
        return datetime.datetime.fromisoformat(as_text(value))
    except ValueError as unreadable:
        raise DecodeError(f"expected a date and a time, got `{as_text(value)}`") from unreadable


def as_date(value: Any) -> datetime.date:
    """A day, as ISO 8601 spells one."""
    try:
        return datetime.date.fromisoformat(as_text(value))
    except ValueError as unreadable:
        raise DecodeError(f"expected a date, got `{as_text(value)}`") from unreadable


def as_json(value: Any) -> Any:
    """A value the document does not describe, which is therefore kept as it arrived."""
    return value


def as_enum(kind: type[StrEnum]) -> Reader[StrEnum]:
    """One of the values a closed list declares."""

    def read_one(value: Any) -> StrEnum:
        try:
            return kind(as_text(value))
        except ValueError as unreadable:
            raise DecodeError(f"`{as_text(value)}` is not one of the values {kind.__name__} declares") from unreadable

    return read_one


def as_list(reader: Reader[Value]) -> Reader[list[Value]]:
    """Every item of an array, each one read the same way."""

    def read_all(value: Any) -> list[Value]:
        if not isinstance(value, list):
            raise DecodeError(f"expected an array, got {type(value).__name__}")
        return [reader(item) for item in value]

    return read_all


def as_map(reader: Reader[Value]) -> Reader[dict[str, Value]]:
    """Every value of an object whose keys the document leaves open."""

    def read_all(value: Any) -> dict[str, Value]:
        if not isinstance(value, dict):
            raise DecodeError(f"expected an object, got {type(value).__name__}")
        return {as_text(key): reader(item) for key, item in value.items()}

    return read_all


def path_segment(value: Any) -> str:
    """A value as one segment of a path, with nothing left in it that could name another one."""
    return urllib.parse.quote(_written(value), safe="")


def query_value(value: Any) -> str:
    """A value as one entry of a query string, escaped when the query is assembled."""
    return _written(value)


def _written(value: Any) -> str:
    """How a value travels in a request line, which is not always how Python prints it."""
    if isinstance(value, bool):
        # `str(True)` is `True`, which no JSON reader on the other end accepts.
        return "true" if value else "false"
    if isinstance(value, StrEnum):
        return value.value
    return str(value)

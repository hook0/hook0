"""What the package reads a JSON document through, and what it refuses to read out of one.

This is the seam the generated half is written against: every member of every schema arrives here,
and the promise the module makes is that a document saying something other than what it declared
stops the read rather than yielding a value whose type annotation lies about what it holds.

The readers are found on the module rather than listed, so one the generator starts emitting for a
shape the API grows is held to the same promise the moment it exists.
"""

from __future__ import annotations

import datetime
import functools
import inspect
import json
import uuid
from enum import StrEnum
from typing import Any

import pytest
from conftest import TEST_TIMEOUT

from hook0 import runtime

pytestmark = pytest.mark.timeout(TEST_TIMEOUT)

# What each reader is offered. Between them they are every shape a JSON document can carry, plus the
# strings that spell a value of a shape JSON has no notion of.
CANDIDATES: tuple[Any, ...] = (
    None,
    True,
    False,
    0,
    12,
    -1,
    1.5,
    "",
    "text",
    "3f2504e0-4f89-41d3-9a0c-0305e82c3301",
    "2026-01-02",
    "2026-01-02T03:04:05+00:00",
    [],
    ["text"],
    {},
    {"a key": "a value"},
)


class AClosedList(StrEnum):
    """A closed list of the shape the generator writes, to hand the reader that reads one."""

    A_VALUE = "a value"


def _readers() -> dict[str, Any]:
    """Every reader the module offers, each one already given whatever it takes to be one.

    A reader takes a document and answers a value. Some are written as a function of one argument
    and are readers already; the rest are written as a function of what they read — a closed list,
    the reader of an item, the reader of a value — and become one once they are given it.
    """
    made: dict[str, Any] = {}
    for name, declared in vars(runtime).items():
        if not name.startswith("as_") or not inspect.isfunction(declared):
            continue
        wanted = list(inspect.signature(declared).parameters.values())
        first = wanted[0]

        if first.name != "value":
            # Written as a function of what it reads rather than of a document: a closed list, or
            # the reader every item or every value is read with.
            if len(wanted) != 1:
                raise AssertionError(f"`{name}` takes {len(wanted)} arguments before it is a reader")
            made[name] = declared(AClosedList if first.annotation.startswith("type[") else runtime.as_text)
            continue

        # Anything a reader takes beside the document names what is being read, for the message it
        # raises; the name itself is immaterial to what it accepts.
        for rest in wanted[1:]:
            if rest.annotation != "str":
                raise AssertionError(f"`{name}` takes a `{rest.name}: {rest.annotation}` nothing here knows to give")
        made[name] = functools.partial(declared, **{rest.name: "the schema under test" for rest in wanted[1:]})
    if not made:
        raise AssertionError("the module offers no reader at all")
    return made


READERS = _readers()

# The one reader that describes nothing, and therefore refuses nothing: it is what a schema declares
# a member with no shape of its own as, and such a member arrives as whatever the API wrote.
KEEPS_ANYTHING = "as_json"


def test_every_reader_either_reads_a_document_or_says_it_could_not() -> None:
    """Whatever arrives, a reader answers a value or raises the one failure a caller catches.

    Nothing else may come out of one: a `TypeError` or an `AttributeError` escaping a reader is a
    member the caller cannot tell apart from a defect in its own code.
    """
    for name, reader in READERS.items():
        for candidate in CANDIDATES:
            try:
                reader(candidate)
            except runtime.DecodeError:
                continue
            except Exception as escaped:  # noqa: BLE001
                raise AssertionError(f"`{name}` let a {type(escaped).__name__} out on {candidate!r}") from escaped


def test_every_reader_but_the_one_that_describes_nothing_refuses_something() -> None:
    """A reader that accepts every shape describes none, which only the opaque one is allowed to."""
    for name, reader in READERS.items():
        refused = [candidate for candidate in CANDIDATES if not _reads(reader, candidate)]
        if name == KEEPS_ANYTHING:
            assert not refused, f"`{name}` is meant to keep whatever arrived, and refused {refused}"
        else:
            assert refused, f"`{name}` accepted every shape a document can carry, so it describes none"


def _reads(reader: Any, candidate: Any) -> bool:
    try:
        reader(candidate)
    except runtime.DecodeError:
        return False
    return True


def test_a_whole_number_is_not_a_boolean_and_a_boolean_is_not_a_whole_number() -> None:
    """`True` is an `int` in Python, and a document that answers one for the other is refused."""
    assert runtime.as_int(12) == 12
    assert runtime.as_bool(True) is True
    with pytest.raises(runtime.DecodeError):
        runtime.as_int(True)
    with pytest.raises(runtime.DecodeError):
        runtime.as_bool(1)


def test_a_number_is_read_whether_or_not_the_document_wrote_a_fractional_part() -> None:
    assert runtime.as_float(1.5) == 1.5
    assert runtime.as_float(2) == 2.0
    assert isinstance(runtime.as_float(2), float)


def test_the_values_a_document_spells_as_text_are_read_back_as_what_they_spell() -> None:
    assert runtime.as_uuid("3f2504e0-4f89-41d3-9a0c-0305e82c3301") == uuid.UUID("3f2504e0-4f89-41d3-9a0c-0305e82c3301")
    assert runtime.as_date("2026-01-02") == datetime.date(2026, 1, 2)
    assert runtime.as_datetime("2026-01-02T03:04:05+00:00") == datetime.datetime(
        2026, 1, 2, 3, 4, 5, tzinfo=datetime.UTC
    )
    assert runtime.as_enum(AClosedList)("a value") is AClosedList.A_VALUE


def test_a_member_a_schema_requires_is_missing_when_the_document_leaves_it_out() -> None:
    with pytest.raises(runtime.DecodeError) as refused:
        runtime.read({}, "a member", runtime.as_text)

    assert "a member" in str(refused.value)


def test_a_member_that_could_not_be_read_says_which_member_it_was() -> None:
    with pytest.raises(runtime.DecodeError) as refused:
        runtime.read({"a member": 12}, "a member", runtime.as_text)

    assert "a member" in str(refused.value)
    assert "string" in str(refused.value)


def test_a_member_a_schema_does_not_require_is_absent_as_readily_as_answered_as_null() -> None:
    assert runtime.maybe({}, "a member", runtime.as_text) is None
    assert runtime.maybe({"a member": None}, "a member", runtime.as_text) is None
    assert runtime.maybe({"a member": "text"}, "a member", runtime.as_text) == "text"


def test_a_member_a_schema_does_not_require_is_still_read_as_what_it_declares() -> None:
    with pytest.raises(runtime.DecodeError) as refused:
        runtime.maybe({"a member": 12}, "a member", runtime.as_text)

    assert "a member" in str(refused.value)


def test_a_body_a_message_quotes_is_cut_at_the_budget_it_is_given() -> None:
    whole = b"x" * (runtime.MAX_PREVIEW_BYTES * 2)

    quoted = runtime.preview(whole)

    assert len(quoted) == runtime.MAX_PREVIEW_BYTES + 1
    assert quoted.endswith("…")
    assert runtime.preview(b"short") == "short"


def test_a_body_above_the_ceiling_is_refused_before_it_is_parsed() -> None:
    above = b'"' + b"x" * runtime.MAX_PAYLOAD_BYTES + b'"'

    with pytest.raises(runtime.DecodeError) as refused:
        runtime.decode_payload(above)

    assert str(runtime.MAX_PAYLOAD_BYTES) in str(refused.value)


def test_a_body_that_is_not_json_is_refused_with_as_much_of_it_as_fits() -> None:
    with pytest.raises(runtime.DecodeError) as refused:
        runtime.decode_payload(b"a gateway wrote this")

    assert "a gateway wrote this" in str(refused.value)

    with pytest.raises(runtime.DecodeError):
        runtime.decode_payload(b"\xff\xfe not utf-8 either")


def test_a_body_that_is_json_is_read_as_the_document_it_carries() -> None:
    assert runtime.decode_payload(json.dumps({"a member": [1, 2]}).encode("utf-8")) == {"a member": [1, 2]}


def test_a_value_travels_in_a_request_line_the_way_the_api_reads_it_back() -> None:
    """Python prints a boolean and a closed list its own way, and neither is what the API reads."""
    assert runtime.query_value(True) == "true"
    assert runtime.query_value(False) == "false"
    assert runtime.query_value(AClosedList.A_VALUE) == "a value"
    assert runtime.query_value(12) == "12"
    assert runtime.query_value(uuid.UUID("3f2504e0-4f89-41d3-9a0c-0305e82c3301")) == (
        "3f2504e0-4f89-41d3-9a0c-0305e82c3301"
    )


def test_a_value_reaching_a_path_can_name_no_segment_the_operation_never_had() -> None:
    assert runtime.path_segment("a value/with a space") == "a%20value%2Fwith%20a%20space"
    assert runtime.path_segment("../../etc/passwd") == "..%2F..%2Fetc%2Fpasswd"
    assert runtime.path_segment(True) == "true"
    assert runtime.path_segment(AClosedList.A_VALUE) == "a%20value"

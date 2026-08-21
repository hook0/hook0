"""What holds for every input, rather than for the ones a case happened to pick.

Three things are checked here. A retry schedule never spends more than the policy that produced it
allows, whichever way the randomness fell. Every generated type reads back what it wrote, for every
value its annotations admit — and the types are discovered rather than listed, so a schema the API
adds is covered the day the generator writes it. And reading a signature header answers with the
one failure this client declares, whatever text reached the endpoint.

The profile is derandomised in `conftest`, so a run that passes here passes in the pipeline too;
the counterexamples that were worth finding are frozen as examples beside the properties they broke.
"""

from __future__ import annotations

import dataclasses
import datetime
import math
import types
import typing
import uuid
from enum import StrEnum
from typing import Any

import pytest
from conftest import TEST_TIMEOUT
from hypothesis import example, given
from hypothesis import strategies as st

import hook0.generated.models as generated
from hook0 import Hook0ClientError, RetryPolicy, Signature, verify_webhook_signature_with_current_time

pytestmark = pytest.mark.timeout(TEST_TIMEOUT)

# How far two sums of the same floats may sit apart before the difference is a defect rather than
# the order they were added in.
ROUNDING = 1e-9

# How deep a generated type may nest before the strategy that builds one gives up. The document
# nests a handful of levels; anything past this is a graph that reads itself.
MAX_NESTING = 8

# A value the API leaves undescribed, which travels as whatever JSON it arrived as.
JSON_VALUES = st.recursive(
    st.none() | st.booleans() | st.integers(min_value=-(2**53), max_value=2**53) | st.text(max_size=16),
    lambda children: st.lists(children, max_size=3) | st.dictionaries(st.text(max_size=8), children, max_size=3),
    max_leaves=6,
)

WHEN = datetime.datetime(2026, 8, 14, 12, 0, 0, tzinfo=datetime.UTC)


def declared_models() -> list[type]:
    """Every class the generator wrote, found by looking at what it wrote.

    Nothing lists the types here: a schema the document adds joins this suite the moment the
    generated module carries it.
    """
    found = [
        member
        for name, member in vars(generated).items()
        if isinstance(member, type) and dataclasses.is_dataclass(member) and not name.startswith("_")
    ]
    return sorted(found, key=lambda model: model.__name__)


def values_of(annotation: Any, depth: int = 0) -> st.SearchStrategy[Any]:
    """Everything a value of that annotation may be."""
    if depth > MAX_NESTING:
        raise AssertionError(f"a generated type nests deeper than the {MAX_NESTING} levels this suite builds")

    origin = typing.get_origin(annotation)
    if origin in (types.UnionType, typing.Union):
        alternatives = [arg for arg in typing.get_args(annotation) if arg is not type(None)]
        drawn = st.one_of([values_of(alternative, depth + 1) for alternative in alternatives])
        if type(None) in typing.get_args(annotation):
            return st.none() | drawn
        return drawn
    if origin is list:
        (item,) = typing.get_args(annotation)
        return st.lists(values_of(item, depth + 1), max_size=3)
    if origin is dict:
        _, value = typing.get_args(annotation)
        return st.dictionaries(st.text(max_size=8), values_of(value, depth + 1), max_size=3)

    if annotation is Any:
        return JSON_VALUES
    if annotation is bool:
        return st.booleans()
    if annotation is int:
        return st.integers(min_value=-(2**63), max_value=2**63 - 1)
    if annotation is float:
        return st.floats(allow_nan=False, allow_infinity=False, width=64)
    if annotation is str:
        return st.text(max_size=32)
    if annotation is uuid.UUID:
        return st.uuids()
    if annotation is datetime.datetime:
        return st.datetimes() | st.datetimes(timezones=st.just(datetime.UTC))
    if annotation is datetime.date:
        return st.dates()
    if isinstance(annotation, type) and issubclass(annotation, StrEnum):
        return st.sampled_from(list(annotation))
    if isinstance(annotation, type) and dataclasses.is_dataclass(annotation):
        return instances_of(annotation, depth + 1)

    raise AssertionError(f"this suite does not know how to build a {annotation!r}")


def instances_of(model: type, depth: int = 0) -> st.SearchStrategy[Any]:
    """Every instance of a generated type its annotations admit."""
    hints = typing.get_type_hints(model)
    members = {field.name: values_of(hints[field.name], depth) for field in dataclasses.fields(model)}
    return st.builds(model, **members)


@pytest.mark.parametrize("model", declared_models(), ids=lambda model: model.__name__)
@given(drawn=st.data())
def test_a_generated_type_reads_back_what_it_wrote(model: type, drawn: st.DataObject) -> None:
    value = drawn.draw(instances_of(model))

    assert model.from_json(value.to_json()) == value


@given(
    max_attempts=st.integers(min_value=-4, max_value=64),
    initial_backoff=st.floats(min_value=0.0, max_value=10.0, allow_nan=False),
    max_backoff=st.floats(min_value=0.0, max_value=10.0, allow_nan=False),
    max_total_delay=st.floats(min_value=0.0, max_value=60.0, allow_nan=False),
    draws=st.lists(st.floats(allow_nan=True, allow_infinity=True), max_size=32),
)
# A policy asking for more attempts than the cap allows, with a budget that lets none of them
# through: the two bounds have to hold together, not one at a time.
@example(
    max_attempts=1000,
    initial_backoff=1.0,
    max_backoff=1.0,
    max_total_delay=0.0,
    draws=[],
)
# Draws that are no draws at all: an unusable source of randomness has to make the client wait the
# whole ceiling, never longer, and never a negative amount of time.
@example(
    max_attempts=8,
    initial_backoff=0.5,
    max_backoff=2.0,
    max_total_delay=60.0,
    draws=[float("nan"), float("inf"), -1.0, 2.0],
)
def test_a_retry_schedule_stays_within_every_bound_of_its_policy(
    max_attempts: int,
    initial_backoff: float,
    max_backoff: float,
    max_total_delay: float,
    draws: list[float],
) -> None:
    policy = RetryPolicy(max_attempts, initial_backoff, max_backoff, max_total_delay)
    delays = policy.delays(draws)

    assert 1 <= policy.attempts() <= 16
    assert len(delays) <= policy.attempts() - 1
    assert math.fsum(delays) <= max(max_total_delay, 0.0) + ROUNDING

    for retry, delay in enumerate(delays, start=1):
        assert delay >= 0.0
        assert delay <= policy.backoff_ceiling(retry) + ROUNDING
        assert delay <= max(max_backoff, 0.0) + ROUNDING

    # The ceiling of a retry never sits below the one before it, so a schedule never hurries up as
    # it goes.
    ceilings = [policy.backoff_ceiling(retry) for retry in range(1, policy.attempts() + 1)]
    assert ceilings == sorted(ceilings)


@given(header=st.text(max_size=256))
# The empty header, which carries neither a moment nor a code.
@example(header="")
# A moment with no code beside it, and a code with no moment.
@example(header="t=1")
@example(header="v0=00")
# A code that is not hexadecimal, which must be refused rather than decoded as far as it goes.
@example(header="t=1,v0=zz")
# A moment Python would happily hold as an integer and no clock could hold as a time.
@example(header="t=" + "9" * 200 + ",v0=00")
# A value carrying the separator the parts are split on, which only the first one counts.
@example(header="t=1,v0=00,h=a=b")
def test_reading_a_signature_answers_with_the_one_failure_this_client_declares(header: str) -> None:
    try:
        Signature.parse(header)
    except Hook0ClientError:
        return


@given(header=st.text(max_size=256))
@example(header="")
@example(header="t=1,v0=zz")
def test_verifying_a_webhook_answers_with_the_one_failure_this_client_declares(header: str) -> None:
    try:
        verify_webhook_signature_with_current_time(header, b"", {}, "secret", 300.0, WHEN)
    except Hook0ClientError:
        return

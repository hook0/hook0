"""What a send does over a real socket, for both flavours of the client.

Each case is written once and run twice, against the blocking client and the awaiting one. What is
asserted is what the API saw — how many requests arrived, and what each of them carried — rather
than what the client was asked to do, so a client that reports success without having sent anything
fails here.
"""

from __future__ import annotations

import dataclasses
import re

import pytest
from conftest import (
    Caller,
    FakeHook0Api,
    ScriptedResponse,
    TEST_TIMEOUT,
    UUID_PATTERN,
    already_ingested,
    an_event,
    ingested,
    prompt_options,
    server_error,
)

from hook0 import Event, Hook0ClientError, Hook0ClientOptions, RetryPolicy

pytestmark = pytest.mark.timeout(TEST_TIMEOUT)

INGESTED_ID = "01961234-5678-7abc-8def-0123456789ab"


def test_a_send_that_succeeds_issues_one_request(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(ingested(INGESTED_ID))

    event_id = caller().send_event(an_event())

    assert event_id == INGESTED_ID
    assert len(api.received) == 1


def test_an_event_carrying_no_id_is_sent_under_one_the_client_generated(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(ingested(INGESTED_ID))

    caller().send_event(an_event())

    # Without an identifier of its own, a repeated request makes Hook0 mint a second one, and the
    # event is ingested and delivered twice.
    assert re.match(UUID_PATTERN, api.event_id_of(0))


def test_an_event_carrying_an_id_is_sent_under_it(api: FakeHook0Api, caller: Caller) -> None:
    chosen = "00000000-0000-0000-0000-000000000000"
    api.will_answer(ingested(chosen))
    event = an_event()
    event.event_id = chosen

    event_id = caller().send_event(event)

    assert event_id == chosen
    assert api.event_id_of(0) == chosen


def test_an_attempt_that_ran_out_of_time_is_repeated_under_the_same_id(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(
        ScriptedResponse(201, ingested(INGESTED_ID).body, held_for=1.0),
        ingested(INGESTED_ID),
    )

    event_id = caller(prompt_options(max_attempts=3, request_timeout=0.2)).send_event(an_event())

    assert event_id == INGESTED_ID
    assert len(api.received) == 2
    # The retry has to repeat the identifier of the attempt it repeats, or Hook0 ingests twice.
    assert api.event_id_of(1) == api.event_id_of(0)
    assert re.match(UUID_PATTERN, api.event_id_of(0))


def test_repeated_server_errors_stop_at_the_configured_number_of_attempts(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(server_error(), server_error(), server_error(), server_error())

    with pytest.raises(Hook0ClientError) as refused:
        caller(prompt_options(max_attempts=3)).send_event(an_event())

    assert "gave up after 3 attempts" in str(refused.value)
    assert len(api.received) == 3


def test_an_answer_the_api_would_repeat_is_not_retried(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(ScriptedResponse(429, {"id": "TooManyEventsToday", "status": 429}))

    # A quota that is spent for the day cannot clear itself between two attempts.
    with pytest.raises(Hook0ClientError) as refused:
        caller(prompt_options(max_attempts=4)).send_event(an_event())

    assert "Sending event" in str(refused.value)
    assert len(api.received) == 1


def test_a_retry_answered_that_the_event_was_already_ingested_reports_success(
    api: FakeHook0Api, caller: Caller
) -> None:
    api.will_answer(server_error(), already_ingested())

    event_id = caller(prompt_options(max_attempts=3)).send_event(an_event())

    # The conflict is the mark of the attempt this one repeats having reached the API.
    assert event_id == api.event_id_of(0)
    assert len(api.received) == 2


def test_a_first_attempt_answered_that_the_event_was_already_ingested_reports_the_conflict(
    api: FakeHook0Api, caller: Caller
) -> None:
    api.will_answer(already_ingested())

    # Nothing this send did can explain the conflict, so the caller has to hear about it.
    with pytest.raises(Hook0ClientError) as refused:
        caller(prompt_options(max_attempts=3)).send_event(an_event())

    assert "EventAlreadyIngested" in str(refused.value)
    assert len(api.received) == 1


def test_a_client_that_does_not_retry_issues_one_request(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(server_error(), server_error(), server_error())

    with pytest.raises(Hook0ClientError):
        caller(Hook0ClientOptions(retry_policy=RetryPolicy.disabled())).send_event(an_event())

    assert len(api.received) == 1


def test_a_payload_above_the_maximum_is_refused_before_any_request(api: FakeHook0Api, caller: Caller) -> None:
    maximum = 16
    api.will_answer(ingested(INGESTED_ID))
    event = Event(
        event_type="auth.user.create",
        payload="x" * (maximum + 1),
        payload_content_type="application/json",
    )

    with pytest.raises(Hook0ClientError) as refused:
        caller(prompt_options(max_payload_bytes=maximum)).send_event(event)

    assert f"{maximum} bytes this client sends at most" in str(refused.value)
    assert api.received == []


def test_a_send_carries_the_application_and_the_credential(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(ingested(INGESTED_ID))

    caller().send_event(an_event())

    request = api.received[0]
    assert request.method == "POST"
    assert request.target.endswith("/event")
    assert request.headers["authorization"] == "Bearer token-xyz"
    assert request.json()["application_id"] == "app-123"
    assert request.json()["labels"] == {"environment": "production"}


def test_event_types_the_application_already_declares_are_not_created(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(
        ScriptedResponse(200, [{"event_type_name": "auth.user.create"}]),
        ScriptedResponse(201, {"event_type_name": "billing.invoice.paid"}),
    )

    created = caller().upsert_event_types(["auth.user.create", "billing.invoice.paid"])

    assert created == ["billing.invoice.paid"]
    assert len(api.received) == 2


@pytest.mark.parametrize(
    "delay",
    [float("inf"), float("nan"), 1e308, -1.0],
    ids=["infinite", "unreadable", "past-what-milliseconds-reach", "backwards"],
)
def test_a_policy_at_the_edges_of_its_type_still_states_four_integers(
    api: FakeHook0Api, caller: Caller, delay: float
) -> None:
    """A delay a float holds but a whole number of milliseconds does not.

    Seconds are a float here and the header states milliseconds as integers, and the two do not
    reach the same places: rounding an infinite or unreadable delay raises rather than producing a
    header, so a policy nobody would configure on purpose took the client down before it opened a
    socket. What has to hold is that the value stays four integers a reader can cut apart.
    """
    api.will_answer(ingested(INGESTED_ID))

    caller(Hook0ClientOptions(retry_policy=RetryPolicy(10**9, delay, delay, delay))).send_event(an_event())

    stated = api.received[0].headers["hook0-client-options"]
    for part in stated.split(","):
        name, _, written = part.partition("=")
        assert written.isdigit(), f"`{name}` states {written!r}, which is no whole number of its own, in {stated!r}"


@pytest.mark.parametrize(
    "value",
    [
        pytest.param(float("inf"), id="infinite"),
        pytest.param(float("-inf"), id="negatively-infinite"),
        pytest.param(float("nan"), id="unreadable"),
    ],
)
@pytest.mark.parametrize("duration", ["initial_backoff", "max_backoff", "max_total_delay"])
def test_a_non_finite_duration_is_read_as_that_fields_default(
    api: FakeHook0Api, caller: Caller, duration: str, value: float
) -> None:
    """A duration that is not a number says nothing about how long to wait, so the default stands.

    Reading it as zero would delete the spacing between attempts and turn a broken policy into a
    burst; reading it as unbounded is what makes a client wait for ever. The default is neither.
    Both halves are held here: what the header states, read off the socket, and what the client
    would actually wait — a header stating a schedule the client does not keep is worse than no
    header, since it reads as fact.
    """
    defaults = RetryPolicy()
    policy = dataclasses.replace(defaults, **{duration: value})
    api.will_answer(ingested(INGESTED_ID))

    caller(Hook0ClientOptions(retry_policy=policy)).send_event(an_event())

    stated = api.received[0].headers["hook0-client-options"]
    expected = (
        f"attempts={defaults.attempts()},"
        f"backoff={round(defaults.initial_backoff * 1000)},"
        f"ceiling={round(defaults.max_backoff * 1000)},"
        f"budget={round(defaults.max_total_delay * 1000)}"
    )
    assert stated == expected, f"a policy whose `{duration}` is {value} states `{stated}`"

    # The schedule the client would keep, at the ends of the draw range and inside it.
    for draws in ([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], [0.5, 0.25, 0.75]):
        assert policy.delays(draws) == defaults.delays(draws), (
            f"a policy whose `{duration}` is {value} waits {policy.delays(draws)} where the "
            f"defaults it states wait {defaults.delays(draws)}"
        )


def test_an_event_type_that_does_not_read_as_three_parts_is_refused(api: FakeHook0Api, caller: Caller) -> None:
    with pytest.raises(Hook0ClientError) as refused:
        caller().upsert_event_types(["not-an-event-type"])

    assert "does not have a valid syntax" in str(refused.value)
    assert api.received == []

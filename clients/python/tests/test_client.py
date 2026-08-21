"""What a send does over a real socket, for both flavours of the client.

Each case is written once and run twice, against the blocking client and the awaiting one. What is
asserted is what the API saw — how many requests arrived, and what each of them carried — rather
than what the client was asked to do, so a client that reports success without having sent anything
fails here.
"""

from __future__ import annotations

import dataclasses
import re
import socket

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


def test_asking_for_no_event_type_at_all_reaches_the_api_for_nothing(api: FakeHook0Api, caller: Caller) -> None:
    assert caller().upsert_event_types([]) == []
    assert api.received == []


def test_an_event_type_the_application_declares_under_another_shape_is_created(
    api: FakeHook0Api, caller: Caller
) -> None:
    """What the API lists is read for the names it carries, and entries carrying none are not names."""
    api.will_answer(
        ScriptedResponse(200, [{"something": "else"}, "not an entry at all", {"event_type_name": 12}]),
        ScriptedResponse(201, {"event_type_name": "auth.user.create"}),
    )

    created = caller().upsert_event_types(["auth.user.create"])

    assert created == ["auth.user.create"]


def test_a_list_of_event_types_that_is_not_a_list_is_reported(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(ScriptedResponse(200, {"event_type_name": "auth.user.create"}))

    with pytest.raises(Hook0ClientError) as refused:
        caller().upsert_event_types(["auth.user.create"])

    assert "did not answer a list of event types" in str(refused.value)
    assert len(api.received) == 1


def test_a_refused_listing_of_event_types_is_reported_with_what_the_api_said(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(ScriptedResponse(403, {"id": "Forbidden", "detail": "this token may not read them"}))

    with pytest.raises(Hook0ClientError) as refused:
        caller().upsert_event_types(["auth.user.create"])

    assert "Getting available event types failed" in str(refused.value)
    assert "this token may not read them" in str(refused.value)
    # Nothing is created off a listing that never arrived.
    assert len(api.received) == 1


def test_a_listing_of_event_types_that_is_not_json_is_reported(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(ScriptedResponse(200, None, verbatim=b"a gateway wrote this, and it is not JSON"))

    with pytest.raises(Hook0ClientError) as refused:
        caller().upsert_event_types(["auth.user.create"])

    assert "did not answer JSON" in str(refused.value)


def test_an_event_type_the_api_refuses_to_create_is_reported_by_name(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(
        ScriptedResponse(200, []),
        ScriptedResponse(409, {"id": "EventTypeAlreadyExist", "detail": "it is already declared"}),
    )

    with pytest.raises(Hook0ClientError) as refused:
        caller().upsert_event_types(["auth.user.create"])

    assert "Creating event type 'auth.user.create' failed" in str(refused.value)
    assert "it is already declared" in str(refused.value)


def test_event_types_cannot_be_read_from_an_api_nothing_is_listening_on(api: FakeHook0Api) -> None:
    """A failure reaching the API is reported as one, rather than as a listing that was empty."""
    with socket.socket() as held:
        held.bind(("127.0.0.1", 0))
        port = held.getsockname()[1]
    api.close()

    for awaiting in (False, True):
        with pytest.raises(Hook0ClientError) as refused:
            Caller(f"http://127.0.0.1:{port}", prompt_options(), awaiting).upsert_event_types(["auth.user.create"])

        assert "Getting available event types failed" in str(refused.value)


def test_an_event_type_cannot_be_created_on_an_api_that_stops_answering(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(ScriptedResponse(200, []))
    asking = caller(prompt_options(max_attempts=1, request_timeout=0.25))

    api.will_answer(ScriptedResponse(201, {}, held_for=5.0))

    with pytest.raises(Hook0ClientError) as refused:
        asking.upsert_event_types(["auth.user.create"])

    assert "Creating event type 'auth.user.create' failed" in str(refused.value)


def test_an_accepted_event_the_api_named_no_id_for_is_not_reported_as_sent(api: FakeHook0Api, caller: Caller) -> None:
    """Repeating it would meet the same answer, so it is given up on rather than retried."""
    api.will_answer(ScriptedResponse(201, {"application_id": "app-123"}))

    with pytest.raises(Hook0ClientError) as refused:
        caller().send_event(an_event())

    assert "without an event id" in str(refused.value)
    assert len(api.received) == 1


@pytest.mark.parametrize(
    "answered",
    [
        ScriptedResponse(201, None, verbatim=b"a gateway wrote this"),
        ScriptedResponse(201, ["an array"]),
        ScriptedResponse(201, {"event_id": 12}),
    ],
    ids=["not-json", "not-an-object", "an-id-that-is-not-text"],
)
def test_an_accepted_event_answered_in_a_shape_this_client_cannot_read_is_not_reported_as_sent(
    api: FakeHook0Api, caller: Caller, answered: ScriptedResponse
) -> None:
    api.will_answer(answered)

    with pytest.raises(Hook0ClientError):
        caller().send_event(an_event())


@pytest.mark.parametrize(
    "answered",
    [
        ScriptedResponse(409, None, verbatim=b"a gateway wrote this"),
        ScriptedResponse(409, ["an array"]),
        ScriptedResponse(409, {"id": 12}),
        ScriptedResponse(409, {}),
    ],
    ids=["not-json", "not-an-object", "a-problem-that-is-not-text", "no-problem-named"],
)
def test_a_conflict_naming_no_problem_this_client_reads_is_not_taken_for_an_earlier_send(
    api: FakeHook0Api, caller: Caller, answered: ScriptedResponse
) -> None:
    """A 409 counts as an event already in only when the body says which problem it was."""
    api.will_answer(answered)

    with pytest.raises(Hook0ClientError):
        caller(prompt_options(max_attempts=1)).send_event(an_event())

    assert len(api.received) == 1


def test_an_event_carrying_metadata_sends_it(api: FakeHook0Api, caller: Caller) -> None:
    api.will_answer(ingested(INGESTED_ID))
    event = an_event()
    event.metadata = {"traced by": "the case"}

    caller().send_event(event)

    assert api.received[0].json()["metadata"] == {"traced by": "the case"}

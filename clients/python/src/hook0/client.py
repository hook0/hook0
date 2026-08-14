"""Sending events to Hook0, idempotently and under bounds the caller can set.

Every event is sent under an identifier this client knows: the one set on the `Event`, or a UUIDv7
it generates when the event carries none. Passing no identifier does not mean the identifier comes
from Hook0 — the value comes from here, travels with the request, and is what `send_event` returns.

That is what makes retrying safe. Hook0 keys events on that identifier, so a request repeated after
a network failure or a server error ingests the event once rather than twice; without a
client-chosen identifier, a repeated request would create a second event and deliver it to every
subscriber. It also gives the answer to a retry its meaning: `EventAlreadyIngested` in reply to a
*repeated* request says an earlier attempt of that same send reached the API, so the send
succeeded. The same answer to a *first* attempt is a genuine conflict and is reported as one.

Only what could end differently is retried: a request that got no answer, and a server error. What
the API refuses outright — a quota that is spent, a payload it will not read — is reported as is,
since repeating it would only spend the same round trip again.

A send is bounded on five axes, each of them the caller's to set: the size of the payload, which is
refused before a socket is opened; how long one attempt is given; how many attempts are made; how
long a single wait between them may be; and how long every wait of one send may add up to.
"""

from __future__ import annotations

import asyncio
import json
import math
import os
import random
import re
import time
import uuid
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from datetime import UTC, datetime
from typing import Any

from .errors import Hook0ClientError
from .transport import (
    AsyncHttpTransport,
    DEFAULT_MAX_RESPONSE_BYTES,
    DEFAULT_REQUEST_TIMEOUT,
    HttpTransport,
    TransportError,
)

# Largest event payload the client agrees to send, in bytes.
#
# Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
# being refused once the JSON envelope around it — metadata, labels, identifiers — is counted. The
# client rules such an event out rather than spending a round trip, and every retry after it, on a
# request that cannot be accepted.
DEFAULT_MAX_PAYLOAD_BYTES = 1024 * 1024

# Most attempts a retry policy can ever make, whatever `max_attempts` says.
#
# A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
# `max_attempts` from turning one send into an unbounded series of requests.
MAX_ATTEMPTS_CAP = 16

# Beyond this many doublings any backoff has long since reached its ceiling.
MAX_BACKOFF_DOUBLINGS = 30

# The identifier Hook0 gives the problem it answers when an event identifier is already taken.
ALREADY_INGESTED = "EventAlreadyIngested"

# Status Hook0 answers when the event identifier a request carries is already taken.
CONFLICT = 409

# First status saying the failure is on Hook0's side, and so could clear on its own.
LOWEST_SERVER_ERROR = 500

# What an event type reads as.
EVENT_TYPE = re.compile(r"^([A-Za-z0-9_]+)[.]([A-Za-z0-9_]+)[.]([A-Za-z0-9_]+)$")

# Where an event is ingested, and where event types are read and created, under the API URL.
EVENT_PATH = "event"
EVENT_TYPES_PATH = "event_types"


@dataclass(frozen=True)
class RetryPolicy:
    """How a client spaces out the attempts of a single send.

    The delay before a retry doubles from `initial_backoff` and is capped by `max_backoff`; the
    delay actually waited is then drawn anywhere between zero and that ceiling, so that emitters
    which failed at the same moment do not come back at the same moment. Retrying stops as soon as
    the delays of the send would add up to more than `max_total_delay`.

    The defaults are four attempts spread over at most five seconds: three retries absorb the blips
    a webhook emitter meets in production — a connection reset, a rolling deployment answering 503 —
    without holding the caller for long, and the five-second budget bounds what the worst send costs
    whatever the individual delays turn out to be.
    """

    # Attempts a single send makes at most, the first one included. `1` disables retrying.
    max_attempts: int = 4
    # Ceiling of the delay before the first retry, in seconds.
    initial_backoff: float = 0.1
    # Ceiling no single delay ever exceeds, in seconds.
    max_backoff: float = 2.0
    # Budget all the delays of one send share, in seconds.
    max_total_delay: float = 5.0

    @staticmethod
    def disabled() -> RetryPolicy:
        """A policy that never retries: one attempt, and the caller hears what it answered."""
        return RetryPolicy(1, 0.0, 0.0, 0.0)

    def attempts(self) -> int:
        """Attempts this policy actually makes: `max_attempts`, brought inside `1..=MAX_ATTEMPTS_CAP`."""
        return min(max(int(self.max_attempts), 1), MAX_ATTEMPTS_CAP)

    def backoff_ceiling(self, retry: int) -> float:
        """Ceiling of the delay before retry number `retry`, where `1` is the first retry.

        It doubles from `initial_backoff` and never exceeds `max_backoff`, so the ceilings of
        successive retries never decrease.
        """
        doublings = min(max(retry - 1, 0), MAX_BACKOFF_DOUBLINGS)
        return min(max(self.initial_backoff, 0.0) * (2**doublings), max(self.max_backoff, 0.0))

    def delays(self, draws: Sequence[float]) -> list[float]:
        """The delays this policy waits between the attempts of one send, one per retry.

        Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as
        soon as the next delay would spend more than `max_total_delay`. There are therefore at most
        `attempts() - 1` delays, and they add up to at most `max_total_delay`.

        A draw that is missing or is not a finite number is read as `1`, which asks for the whole
        ceiling: an unusable source of randomness makes the client wait longer, never less.
        """
        budget = max(self.max_total_delay, 0.0)
        waits: list[float] = []
        spent = 0.0

        for retry in range(1, self.attempts()):
            delay = self.backoff_ceiling(retry) * _draw(draws, retry - 1)
            if spent + delay > budget:
                break
            spent += delay
            waits.append(delay)

        return waits


def _draw(draws: Sequence[float], index: int) -> float:
    """The draw for one retry, brought back inside `[0, 1]` whatever the randomness gave."""
    if index >= len(draws):
        return 1.0
    drawn = draws[index]
    if not math.isfinite(drawn):
        return 1.0
    return min(max(float(drawn), 0.0), 1.0)


def _jitter_draws(count: int) -> list[float]:
    """Draws used to jitter the delays of one send.

    Jitter only has to keep emitters that failed together from coming back together; it does not
    have to be unpredictable, so the platform's own generator is enough.
    """
    return [random.random() for _ in range(max(count, 0))]  # noqa: S311


@dataclass(frozen=True)
class Hook0ClientOptions:
    """Every bound a client applies to one send."""

    retry_policy: RetryPolicy = field(default_factory=RetryPolicy)
    request_timeout: float = DEFAULT_REQUEST_TIMEOUT
    max_payload_bytes: int = DEFAULT_MAX_PAYLOAD_BYTES
    max_response_bytes: int = DEFAULT_MAX_RESPONSE_BYTES


# The bounds a client applies when the caller names none, built once so that naming none is not a
# call made every time a client is built.
DEFAULT_OPTIONS = Hook0ClientOptions()


@dataclass
class Event:
    """An event to send to Hook0.

    `event_id` is the caller's to set when it already has one to key the event on. Left unset, the
    client generates a UUIDv7, sends it and returns it — which is what lets it repeat a request
    without risking a second copy of the event being ingested and delivered to every subscriber.
    """

    event_type: str
    payload: str
    payload_content_type: str
    labels: Mapping[str, str] = field(default_factory=dict)
    metadata: Mapping[str, str] | None = None
    occurred_at: datetime | None = None
    event_id: str | None = None


@dataclass(frozen=True)
class EventType:
    """An event type, read out of the `service.resource_type.verb` it is written as."""

    service: str
    resource_type: str
    verb: str

    @staticmethod
    def parse(written: str) -> EventType:
        """Reads an event type, refusing one that does not name all three of its parts."""
        read = EVENT_TYPE.match(written)
        if read is None:
            raise Hook0ClientError.invalid_event_type(written)
        return EventType(read.group(1), read.group(2), read.group(3))

    def __str__(self) -> str:
        return f"{self.service}.{self.resource_type}.{self.verb}"


def generate_event_id() -> str:
    """A UUIDv7, the shape of identifier Hook0 mints when it is the one choosing.

    Its leading 48 bits are the current time in milliseconds, so identifiers generated in sequence
    are ordered, which is what keeps the index they end up in from being written all over.
    """
    milliseconds = int(time.time() * 1000)
    drawn = bytearray(os.urandom(16))
    drawn[0:6] = milliseconds.to_bytes(6, "big")
    drawn[6] = (drawn[6] & 0x0F) | 0x70
    drawn[8] = (drawn[8] & 0x3F) | 0x80
    return str(uuid.UUID(bytes=bytes(drawn)))


def _event_id_of(event: Event) -> str:
    """The identifier an event is sent under: the one it carries, or one generated for it."""
    if isinstance(event.event_id, str) and event.event_id:
        return event.event_id
    return generate_event_id()


def _full_event(event: Event, application_id: str, event_id: str) -> dict[str, Any]:
    """An event as the API reads one."""
    occurred_at = event.occurred_at if event.occurred_at is not None else datetime.now(UTC)
    written: dict[str, Any] = {
        "event_id": event_id,
        "application_id": application_id,
        "event_type": event.event_type,
        "payload": event.payload,
        "payload_content_type": event.payload_content_type,
        "occurred_at": occurred_at.isoformat(),
        "labels": dict(event.labels),
    }
    if event.metadata is not None:
        written["metadata"] = dict(event.metadata)
    return written


@dataclass(frozen=True)
class _Attempt:
    """What one attempt at sending an event ended with."""

    # The identifier the API ingested the event under, set only when it did.
    ingested: str | None
    # Whether the API refused the event because it already holds one under the same identifier.
    already_ingested: bool
    # What went wrong, in the words a caller is given.
    detail: str
    # Whether repeating this very request could end differently.
    retryable: bool


def _ingested(event_id: str) -> _Attempt:
    return _Attempt(event_id, False, "", False)


def _conflicted(detail: str) -> _Attempt:
    return _Attempt(None, True, detail, False)


def _failed(detail: str, retryable: bool) -> _Attempt:
    return _Attempt(None, False, detail, retryable)


def _read_attempt(status: int, payload: bytes) -> _Attempt:
    """What the API answered one attempt, and whether repeating it could end differently."""
    body = payload.decode("utf-8", errors="replace")

    if 200 <= status < 300:
        ingested = _ingested_id(body)
        if ingested is None:
            # The API accepted the event but answered something this client cannot read; repeating
            # the request would meet the same answer.
            return _failed(f"Hook0 answered {status} without an event id", False)
        return _ingested(ingested)

    if status == CONFLICT and _is_already_ingested(body):
        return _conflicted(body)

    # Only the server side of a server error can change between two identical requests.
    return _failed(body, status >= LOWEST_SERVER_ERROR)


def _ingested_id(body: str) -> str | None:
    """The identifier the API says it ingested the event under."""
    try:
        answered = json.loads(body)
    except ValueError:
        return None
    if not isinstance(answered, dict):
        return None
    ingested = answered.get("event_id")
    if isinstance(ingested, str):
        return ingested
    return None


def _is_already_ingested(body: str) -> bool:
    """Whether a problem document is the one Hook0 answers when an identifier is already taken."""
    try:
        problem = json.loads(body)
    except ValueError:
        return False
    return isinstance(problem, dict) and problem.get("id") == ALREADY_INGESTED


def _refuse_oversized(event: Event, event_id: str, maximum: int) -> None:
    """Rules an oversized payload out, before a socket is opened for it."""
    size = len(event.payload.encode("utf-8"))
    if size > maximum:
        raise Hook0ClientError.payload_too_large(event_id, size, maximum)


def _given_up(event_id: str, attempts: int, waited: float, detail: str) -> Hook0ClientError:
    """What to raise when a send is being given up on."""
    if attempts <= 1:
        return Hook0ClientError.event_sending(event_id, detail)
    return Hook0ClientError.retries_exhausted(event_id, attempts, waited, detail)


def _declared_names(answered: Any) -> set[str]:
    """The event types the application already declares, out of what the API listed."""
    if not isinstance(answered, list):
        raise Hook0ClientError.available_event_types("the API did not answer a list of event types")

    declared = set()
    for entry in answered:
        if isinstance(entry, dict) and isinstance(entry.get("event_type_name"), str):
            declared.add(entry["event_type_name"])
    return declared


class Hook0Client:
    """The Hook0 client.

    Built once and shared wherever an application sends events.

    - `api_url`: base API URL of a Hook0 instance, such as `https://app.hook0.com/api/v1`.
    - `application_id`: identifier of the Hook0 application events are sent to.
    - `token`: an authentication token valid for that application.
    - `options`: the bounds one send is held to.
    """

    def __init__(
        self,
        api_url: str,
        application_id: str,
        token: str,
        options: Hook0ClientOptions = DEFAULT_OPTIONS,
    ) -> None:
        self.api_url = api_url
        self.application_id = application_id
        self.options = options
        self.transport = HttpTransport(
            api_url,
            token,
            timeout=options.request_timeout,
            max_response_bytes=options.max_response_bytes,
        )

    def send_event(self, event: Event) -> str:
        """Sends an event, and answers the identifier it was sent under."""
        event_id = _event_id_of(event)
        _refuse_oversized(event, event_id, self.options.max_payload_bytes)

        body = _full_event(event, self.application_id, event_id)
        policy = self.options.retry_policy
        delays = policy.delays(_jitter_draws(policy.attempts() - 1))

        attempt = 0
        waited = 0.0
        while True:
            attempt += 1
            outcome = self._attempt(body)

            if outcome.ingested is not None:
                return outcome.ingested
            if outcome.already_ingested:
                if attempt > 1:
                    return event_id
                raise Hook0ClientError.event_sending(event_id, outcome.detail)

            retry = attempt - 1
            if outcome.retryable and retry < len(delays):
                time.sleep(delays[retry])
                waited += delays[retry]
                continue

            raise _given_up(event_id, attempt, waited, outcome.detail)

    def _attempt(self, body: Mapping[str, Any]) -> _Attempt:
        """One attempt at sending an already-bounded event."""
        try:
            status, payload = self.transport.request("POST", EVENT_PATH, [], body)
        except TransportError as unreachable:
            return _failed(str(unreachable), True)
        return _read_attempt(status, payload)

    def upsert_event_types(self, event_types: Sequence[str]) -> list[str]:
        """Creates the event types the application does not declare yet, and answers those."""
        wanted = [EventType.parse(written) for written in event_types]
        if not wanted:
            return []

        try:
            status, payload = self.transport.request("GET", EVENT_TYPES_PATH, [("application_id", self.application_id)])
        except TransportError as unreachable:
            raise Hook0ClientError.available_event_types(str(unreachable)) from unreachable
        declared = _declared_event_types(status, payload)

        created = []
        for event_type in wanted:
            if str(event_type) in declared:
                continue
            self._create_event_type(event_type)
            created.append(str(event_type))
        return created

    def _create_event_type(self, event_type: EventType) -> None:
        """Declares one event type on the application."""
        body = _event_type_body(event_type, self.application_id)
        try:
            status, payload = self.transport.request("POST", EVENT_TYPES_PATH, [], body)
        except TransportError as unreachable:
            raise Hook0ClientError.creating_event_type(str(event_type), str(unreachable)) from unreachable
        _refuse_creation(event_type, status, payload)


class Hook0AsyncClient:
    """The Hook0 client, for an application that awaits what it does.

    Every bound, every answer and every reason to retry is the one `Hook0Client` applies: the two
    differ in how they wait, and in nothing else.
    """

    def __init__(
        self,
        api_url: str,
        application_id: str,
        token: str,
        options: Hook0ClientOptions = DEFAULT_OPTIONS,
    ) -> None:
        self.api_url = api_url
        self.application_id = application_id
        self.options = options
        self.transport = AsyncHttpTransport(
            api_url,
            token,
            timeout=options.request_timeout,
            max_response_bytes=options.max_response_bytes,
        )

    async def send_event(self, event: Event) -> str:
        """Sends an event, and answers the identifier it was sent under."""
        event_id = _event_id_of(event)
        _refuse_oversized(event, event_id, self.options.max_payload_bytes)

        body = _full_event(event, self.application_id, event_id)
        policy = self.options.retry_policy
        delays = policy.delays(_jitter_draws(policy.attempts() - 1))

        attempt = 0
        waited = 0.0
        while True:
            attempt += 1
            outcome = await self._attempt(body)

            if outcome.ingested is not None:
                return outcome.ingested
            if outcome.already_ingested:
                if attempt > 1:
                    return event_id
                raise Hook0ClientError.event_sending(event_id, outcome.detail)

            retry = attempt - 1
            if outcome.retryable and retry < len(delays):
                await asyncio.sleep(delays[retry])
                waited += delays[retry]
                continue

            raise _given_up(event_id, attempt, waited, outcome.detail)

    async def _attempt(self, body: Mapping[str, Any]) -> _Attempt:
        """One attempt at sending an already-bounded event."""
        try:
            status, payload = await self.transport.request("POST", EVENT_PATH, [], body)
        except TransportError as unreachable:
            return _failed(str(unreachable), True)
        return _read_attempt(status, payload)

    async def upsert_event_types(self, event_types: Sequence[str]) -> list[str]:
        """Creates the event types the application does not declare yet, and answers those."""
        wanted = [EventType.parse(written) for written in event_types]
        if not wanted:
            return []

        try:
            status, payload = await self.transport.request(
                "GET", EVENT_TYPES_PATH, [("application_id", self.application_id)]
            )
        except TransportError as unreachable:
            raise Hook0ClientError.available_event_types(str(unreachable)) from unreachable
        declared = _declared_event_types(status, payload)

        created = []
        for event_type in wanted:
            if str(event_type) in declared:
                continue
            await self._create_event_type(event_type)
            created.append(str(event_type))
        return created

    async def _create_event_type(self, event_type: EventType) -> None:
        """Declares one event type on the application."""
        body = _event_type_body(event_type, self.application_id)
        try:
            status, payload = await self.transport.request("POST", EVENT_TYPES_PATH, [], body)
        except TransportError as unreachable:
            raise Hook0ClientError.creating_event_type(str(event_type), str(unreachable)) from unreachable
        _refuse_creation(event_type, status, payload)


def _declared_event_types(status: int, payload: bytes) -> set[str]:
    """The event types an application already declares, out of what the API answered."""
    if not 200 <= status < 300:
        raise Hook0ClientError.available_event_types(payload.decode("utf-8", errors="replace"))
    try:
        answered = json.loads(payload)
    except ValueError as unreadable:
        raise Hook0ClientError.available_event_types("the API did not answer JSON") from unreadable
    return _declared_names(answered)


def _event_type_body(event_type: EventType, application_id: str) -> dict[str, Any]:
    """An event type as the API creates one."""
    return {
        "application_id": application_id,
        "service": event_type.service,
        "resource_type": event_type.resource_type,
        "verb": event_type.verb,
    }


def _refuse_creation(event_type: EventType, status: int, payload: bytes) -> None:
    """Reports an event type the API would not create."""
    if not 200 <= status < 300:
        raise Hook0ClientError.creating_event_type(str(event_type), payload.decode("utf-8", errors="replace"))

"""A Hook0 API on a loopback port, and the two ways of asking a client to reach it.

Every case below goes over a real socket: the request the client builds, the headers it sets, the
way it reads an answer and the way it gives up on one are all the real ones. Nothing here stands in
for a part of the client, so a case that passes says the client works rather than that it was
called.

The same cases run against both flavours of the client. They differ in how they wait and in nothing
else, so a case written once and run twice is what keeps them from drifting apart.
"""

from __future__ import annotations

import asyncio
import json
import threading
import time
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any

import pytest
from hypothesis import HealthCheck, settings

from hook0 import Event, Hook0AsyncClient, Hook0Client, Hook0ClientOptions, RetryPolicy

# The property suites are derandomised: a run that passes here passes in the pipeline, and a
# failure there is one somebody can reproduce rather than one that goes away on a retry. The
# deadline is off because the machines a pipeline runs on are not the machine a property was
# written on, and a slow first draw is not a defect.
settings.register_profile(
    "hook0",
    derandomize=True,
    deadline=None,
    max_examples=50,
    suppress_health_check=[HealthCheck.function_scoped_fixture],
)
settings.load_profile("hook0")

# No request a case makes is anywhere near this large; the cap bounds what one connection buffers.
MAX_REQUEST_BODY_BYTES = 64 * 1024

# Every case talks to a loopback socket, so none of them has a reason to take this long.
TEST_TIMEOUT = 20

# The shape a UUID has, whichever version it carries.
UUID_PATTERN = r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"


@dataclass
class ScriptedResponse:
    """What the API answers to one request, in the order the case scripted it."""

    status: int
    body: Any
    # How long the API sits on the answer before writing anything, in seconds.
    held_for: float = 0.0
    # What the answer carries beside its body, such as the delay a paced instance names.
    headers: dict[str, str] = field(default_factory=dict)
    # The body as bytes, for the cases whose point is that it is not a JSON document at all.
    verbatim: bytes | None = None


@dataclass
class ReceivedRequest:
    """A request the API received, in the order it received it."""

    method: str
    target: str
    headers: dict[str, str]
    body: str

    def json(self) -> Any:
        return json.loads(self.body)


@dataclass
class FakeHook0Api:
    """A Hook0 API listening on a loopback port for the lifetime of one case."""

    received: list[ReceivedRequest] = field(default_factory=list)
    _scripted: list[ScriptedResponse] = field(default_factory=list)
    _answered: int = 0
    _lock: threading.Lock = field(default_factory=threading.Lock)
    _server: ThreadingHTTPServer | None = None
    _thread: threading.Thread | None = None

    def will_answer(self, *responses: ScriptedResponse) -> None:
        """Queues the answers the case expects the client to draw, in order."""
        with self._lock:
            self._scripted.extend(responses)

    def next_response(self) -> ScriptedResponse:
        with self._lock:
            if self._answered >= len(self._scripted):
                return ScriptedResponse(500, {"error": "the case scripted no answer for this request"})
            scripted = self._scripted[self._answered]
            self._answered += 1
            return scripted

    def record(self, request: ReceivedRequest) -> None:
        with self._lock:
            self.received.append(request)

    def event_id_of(self, index: int) -> str:
        """The event identifier request number `index` carried, as the API read it."""
        if index >= len(self.received):
            raise AssertionError(f"expected at least {index + 1} requests, got {len(self.received)}")
        body = self.received[index].json()
        return body["event_id"]

    def listen(self) -> None:
        self._server = ThreadingHTTPServer(("127.0.0.1", 0), _handler_for(self))
        self._server.daemon_threads = True
        # The accept loop is polled often enough that tearing one API down between two cases costs
        # a fraction of what one case takes, rather than half a second each time.
        self._thread = threading.Thread(target=self._server.serve_forever, kwargs={"poll_interval": 0.02}, daemon=True)
        self._thread.start()

    @property
    def base_url(self) -> str:
        if self._server is None:
            raise AssertionError("the fake Hook0 API is not listening")
        host, port = self._server.server_address[:2]
        return f"http://{host}:{port}"

    def close(self) -> None:
        if self._server is not None:
            self._server.shutdown()
            self._server.server_close()
        if self._thread is not None:
            self._thread.join(timeout=TEST_TIMEOUT)


def _handler_for(api: FakeHook0Api) -> type[BaseHTTPRequestHandler]:
    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, *_args: Any) -> None:
            """Kept quiet: a case reports what it observed, not what the server printed."""

        def _serve(self) -> None:
            length = int(self.headers.get("Content-Length", "0"))
            if length > MAX_REQUEST_BODY_BYTES:
                self.send_error(413)
                return
            body = self.rfile.read(length).decode("utf-8") if length else ""
            api.record(
                ReceivedRequest(
                    method=self.command,
                    target=self.path,
                    headers={name.lower(): value for name, value in self.headers.items()},
                    body=body,
                )
            )

            scripted = api.next_response()
            if scripted.held_for:
                time.sleep(scripted.held_for)

            answer = scripted.verbatim if scripted.verbatim is not None else json.dumps(scripted.body).encode("utf-8")
            try:
                self.send_response(scripted.status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(answer)))
                self.send_header("Connection", "close")
                for name, value in scripted.headers.items():
                    self.send_header(name, value)
                self.end_headers()
                self.wfile.write(answer)
            except OSError:
                # The client gave up waiting and closed the connection, which is the very thing a
                # held answer is scripted to make it do.
                pass

        do_GET = _serve
        do_POST = _serve
        do_PUT = _serve
        do_DELETE = _serve

    return Handler


@pytest.fixture
def api() -> Any:
    """A Hook0 API listening for the lifetime of one case."""
    running = FakeHook0Api()
    running.listen()
    try:
        yield running
    finally:
        running.close()


class Caller:
    """One of the two clients, asked the same things in the same words."""

    def __init__(self, base_url: str, options: Hook0ClientOptions, awaiting: bool) -> None:
        self.awaiting = awaiting
        self.client: Any = (
            Hook0AsyncClient(base_url, "app-123", "token-xyz", options)
            if awaiting
            else Hook0Client(base_url, "app-123", "token-xyz", options)
        )

    def send_event(self, event: Event) -> str:
        if self.awaiting:
            return asyncio.run(self.client.send_event(event))
        return self.client.send_event(event)

    def upsert_event_types(self, event_types: list[str]) -> list[str]:
        if self.awaiting:
            return asyncio.run(self.client.upsert_event_types(event_types))
        return self.client.upsert_event_types(event_types)


@pytest.fixture(params=[False, True], ids=["sync", "async"])
def awaiting(request: Any) -> bool:
    """Which flavour of the client a case is running against."""
    return request.param


@pytest.fixture
def caller(api: FakeHook0Api, awaiting: bool) -> Any:
    """A client built on the default bounds, for the case that needs no others."""

    def build(options: Hook0ClientOptions | None = None) -> Caller:
        return Caller(api.base_url, options if options is not None else prompt_options(), awaiting)

    return build


def prompt_retries(max_attempts: int = 4) -> RetryPolicy:
    """A schedule short enough that a case spends its time on requests rather than on waiting.

    Its budget sits far above what its delays add up to, so the number of attempts a case observes
    is the one its policy asked for rather than the one its budget allowed.
    """
    return RetryPolicy(max_attempts, 0.005, 0.005, 1.0)


def prompt_options(max_attempts: int = 4, request_timeout: float = 5.0, **bounds: Any) -> Hook0ClientOptions:
    return Hook0ClientOptions(retry_policy=prompt_retries(max_attempts), request_timeout=request_timeout, **bounds)


def an_event() -> Event:
    return Event(
        event_type="auth.user.create",
        payload='{"email": "test@example.com"}',
        payload_content_type="application/json",
        labels={"environment": "production"},
    )


def ingested(event_id: str) -> ScriptedResponse:
    return ScriptedResponse(201, {"application_id": "app-123", "event_id": event_id, "received_at": "2026-01-01"})


def already_ingested() -> ScriptedResponse:
    return ScriptedResponse(
        409,
        {
            "id": "EventAlreadyIngested",
            "title": "Event already Ingested",
            "detail": "This event was previously ingested and recorded inside Hook0 service.",
            "status": 409,
            "type": "https://documentation.hook0.com/problems",
        },
    )


def server_error() -> ScriptedResponse:
    return ScriptedResponse(500, {"id": "InternalServerError", "status": 500})

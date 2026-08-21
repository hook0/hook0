"""What a transport does with an answer that is not one, and with a URL nothing can be sent to.

Every case here writes bytes onto a real socket and lets a real transport read them: the answers are
malformed on purpose — a status line that is not one, a length that is not a number, a chunk header
in no base, a head above what a caller agrees to hold, a connection that stops mid-answer — because
that is the half of a transport a well-behaved API never exercises and a broken or hostile one does
every time.

Nothing stands in for a part of either transport. What a case observes is what an application would.
"""

from __future__ import annotations

import asyncio
import socket
import socketserver
import threading
from collections.abc import Callable, Iterator
from typing import Any

import pytest
from conftest import TEST_TIMEOUT

from hook0.transport import (
    AsyncHttpTransport,
    HttpTransport,
    MAX_CHUNKS,
    MAX_HEADERS,
    MAX_HEAD_BYTES,
    TransportError,
)

pytestmark = pytest.mark.timeout(TEST_TIMEOUT)

# No request a case makes is anywhere near this large; the cap bounds what one connection buffers.
MAX_REQUEST_BYTES = 64 * 1024

# What the answers below are read under, small enough that crossing it costs a case a few bytes.
A_SMALL_CEILING = 512

# A host no resolver can be asked about: a label above what a name may carry, which is refused where
# the name is encoded rather than by anything on the network.
AN_UNRESOLVABLE_HOST = "a" * 300

# What a case hands the server when the answer it is after is the one that never comes.
HOLD = object()


class _Handler(socketserver.StreamRequestHandler):
    """Reads one request off the connection, writes back whatever the case wrote, and closes."""

    def handle(self) -> None:
        head = b""
        while b"\r\n\r\n" not in head and len(head) < MAX_REQUEST_BYTES:
            read = self.rfile.read(1)
            if not read:
                return
            head += read

        length = 0
        for line in head.split(b"\r\n"):
            name, _, value = line.partition(b":")
            if name.strip().lower() == b"content-length":
                length = min(int(value.strip()), MAX_REQUEST_BYTES)
        if length:
            self.rfile.read(length)

        self.server.received.append(head)  # type: ignore[attr-defined]
        if self.server.answer is HOLD:  # type: ignore[attr-defined]
            # Accepted and never answered, which is what a caller's timeout is for. Bounded all the
            # same, so that a case that never gives up cannot hold the run either.
            self.server.holding.wait(TEST_TIMEOUT)  # type: ignore[attr-defined]
            return
        try:
            self.wfile.write(self.server.answer)  # type: ignore[attr-defined]
        except OSError:
            # The transport gave up on the answer and closed, which several cases below script.
            pass


class _RawApi(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True

    def __init__(self, answer: bytes | object) -> None:
        super().__init__(("127.0.0.1", 0), _Handler)
        self.answer = answer
        self.received: list[bytes] = []
        self.holding = threading.Event()

    @property
    def base_url(self) -> str:
        host, port = self.server_address[:2]
        return f"http://{host}:{port}"


@pytest.fixture
def answering() -> Iterator[Callable[[bytes], _RawApi]]:
    """A loopback port answering exactly the bytes a case hands it, for the lifetime of that case."""
    running: list[_RawApi] = []

    def listen(answer: bytes) -> _RawApi:
        server = _RawApi(answer)
        threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.02}, daemon=True).start()
        running.append(server)
        return server

    yield listen

    for server in running:
        server.holding.set()
        server.shutdown()
        server.server_close()


def _awaiting(base_url: str, **bounds: Any) -> Callable[[], tuple[int, dict[str, str], bytes]]:
    """The awaiting transport, which is the one that reads an answer off the wire itself.

    The blocking one is handed a parsed head and a decoded body by the standard library, so the
    cases below that are about how bytes are turned into an answer are about this one alone. What
    the two share — the ceilings, the URLs neither can be sent to, what a refusal is — is asked of
    both through `reaching`.
    """
    transport = AsyncHttpTransport(base_url, "token-xyz", **bounds)

    def deliver() -> tuple[int, dict[str, str], bytes]:
        return asyncio.run(transport.deliver("GET", "/api/v1/instance/", []))

    return deliver


@pytest.fixture(params=[False, True], ids=["blocking", "awaiting"])
def reaching(request: Any) -> Callable[..., Any]:
    """One of the two transports, asked the same thing in the same words."""

    def ask(base_url: str, **bounds: Any) -> Callable[[], tuple[int, dict[str, str], bytes]]:
        awaiting = request.param
        transport = (
            AsyncHttpTransport(base_url, "token-xyz", **bounds)
            if awaiting
            else HttpTransport(base_url, "token-xyz", **bounds)
        )

        def deliver() -> tuple[int, dict[str, str], bytes]:
            answered = transport.deliver("GET", "/api/v1/instance/", [])
            if awaiting:
                return asyncio.run(answered)
            return answered

        return deliver

    return ask


def _answer(head: str, body: bytes = b"") -> bytes:
    return head.replace("\n", "\r\n").encode("utf-8") + body


def test_an_answer_this_client_can_read_is_read(answering: Any, reaching: Any) -> None:
    served = answering(_answer("HTTP/1.1 200 OK\nContent-Length: 2\nX-Answered-By: the case\n\n", b"{}"))

    status, headers, payload = reaching(served.base_url)()

    assert status == 200
    assert payload == b"{}"
    # Read back under the name a caller looks it up by, whichever case the server wrote it in.
    assert headers["x-answered-by"] == "the case"


def test_an_answer_written_as_chunks_is_read_back_whole(answering: Any, reaching: Any) -> None:
    served = answering(
        _answer(
            "HTTP/1.1 200 OK\nTransfer-Encoding: chunked\n\n",
            b'5\r\n{"a":\r\n3\r\n 1}\r\n0\r\n\r\n',
        )
    )

    status, _, payload = reaching(served.base_url)()

    assert status == 200
    assert payload == b'{"a": 1}'


def test_an_answer_of_unstated_length_runs_to_the_end_of_the_connection(answering: Any, reaching: Any) -> None:
    served = answering(_answer("HTTP/1.1 200 OK\n\n", b'{"read": "to the close"}'))

    status, _, payload = reaching(served.base_url)()

    assert status == 200
    assert payload == b'{"read": "to the close"}'


def test_an_answer_that_does_not_open_with_a_status_line_is_refused(answering: Any, reaching: Any) -> None:
    served = answering(_answer("a proxy wrote this instead of an answer\n\n"))

    with pytest.raises(TransportError) as refused:
        reaching(served.base_url)()

    # Reading it again draws the same thing, so there is nothing to gain from repeating the request.
    assert refused.value.transient is False


def test_a_status_that_is_not_a_number_is_refused(answering: Any, reaching: Any) -> None:
    served = answering(_answer("HTTP/1.1 OK\nContent-Length: 0\n\n"))

    with pytest.raises(TransportError) as refused:
        reaching(served.base_url)()

    assert refused.value.transient is False


def test_a_length_that_is_not_a_length_is_refused(answering: Any) -> None:
    for announced in ("not a number", "-1"):
        served = answering(_answer(f"HTTP/1.1 200 OK\nContent-Length: {announced}\n\n", b"{}"))

        with pytest.raises(TransportError) as refused:
            _awaiting(served.base_url)()

        assert refused.value.transient is False


def test_a_body_that_stops_before_the_length_it_announced_can_be_met_again(answering: Any) -> None:
    """The one protocol failure worth repeating: the next answer could carry the whole of it."""
    served = answering(_answer("HTTP/1.1 200 OK\nContent-Length: 64\n\n", b"{}"))

    with pytest.raises(TransportError) as refused:
        _awaiting(served.base_url)()

    assert refused.value.transient is True


def test_a_connection_that_stops_mid_head_can_be_met_again(answering: Any) -> None:
    served = answering(_answer("HTTP/1.1 200 OK\nContent-Len"))

    with pytest.raises(TransportError) as refused:
        _awaiting(served.base_url)()

    assert refused.value.transient is True


def test_a_chunk_header_in_no_base_is_refused(answering: Any) -> None:
    served = answering(_answer("HTTP/1.1 200 OK\nTransfer-Encoding: chunked\n\n", b"not a size\r\n{}\r\n0\r\n\r\n"))

    with pytest.raises(TransportError) as refused:
        _awaiting(served.base_url)()

    assert refused.value.transient is False


def test_an_answer_announcing_more_than_the_ceiling_is_refused_before_it_is_read(answering: Any, reaching: Any) -> None:
    served = answering(
        _answer(f"HTTP/1.1 200 OK\nContent-Length: {A_SMALL_CEILING * 4}\n\n", b"x" * (A_SMALL_CEILING * 4))
    )

    with pytest.raises(TransportError) as refused:
        reaching(served.base_url, max_response_bytes=A_SMALL_CEILING)()

    assert refused.value.transient is False
    assert str(A_SMALL_CEILING) in str(refused.value)


def test_an_answer_of_unstated_length_above_the_ceiling_is_refused(answering: Any, reaching: Any) -> None:
    served = answering(_answer("HTTP/1.1 200 OK\n\n", b"x" * (A_SMALL_CEILING * 4)))

    with pytest.raises(TransportError) as refused:
        reaching(served.base_url, max_response_bytes=A_SMALL_CEILING)()

    assert refused.value.transient is False


def test_chunks_adding_up_to_more_than_the_ceiling_are_refused(answering: Any) -> None:
    chunk = b"x" * A_SMALL_CEILING
    body = b"".join(f"{A_SMALL_CEILING:x}\r\n".encode() + chunk + b"\r\n" for _ in range(4)) + b"0\r\n\r\n"
    served = answering(_answer("HTTP/1.1 200 OK\nTransfer-Encoding: chunked\n\n", body))

    with pytest.raises(TransportError) as refused:
        _awaiting(served.base_url, max_response_bytes=A_SMALL_CEILING)()

    assert refused.value.transient is False


def test_more_chunks_than_are_read_at_most_are_refused(answering: Any) -> None:
    """Only the awaiting transport reads chunks itself; the blocking one is handed a body already."""
    body = b"1\r\nx\r\n" * (MAX_CHUNKS + 1) + b"0\r\n\r\n"
    served = _RawApi(_answer("HTTP/1.1 200 OK\nTransfer-Encoding: chunked\n\n", body))
    threading.Thread(target=served.serve_forever, kwargs={"poll_interval": 0.02}, daemon=True).start()

    try:
        with pytest.raises(TransportError) as refused:
            asyncio.run(AsyncHttpTransport(served.base_url, "token-xyz").deliver("GET", "/api/v1/instance/", []))
    finally:
        served.shutdown()
        served.server_close()

    assert refused.value.transient is False
    assert str(MAX_CHUNKS) in str(refused.value)


def test_a_head_above_what_a_caller_agrees_to_hold_is_refused(answering: Any, reaching: Any) -> None:
    """Every line counted together, which is the bound that says what a head costs at most."""
    padding = "x" * 500
    lines = "".join(f"X-Padding-{index}: {padding}\n" for index in range(MAX_HEADERS - 4))
    served = answering(_answer(f"HTTP/1.1 200 OK\nContent-Length: 2\n{lines}\n", b"{}"))

    with pytest.raises(TransportError) as refused:
        reaching(served.base_url)()

    assert refused.value.transient is False
    assert str(MAX_HEAD_BYTES) in str(refused.value)


def test_more_headers_than_are_read_at_most_are_refused(answering: Any, reaching: Any) -> None:
    lines = "".join(f"X-Padding-{index}: a\n" for index in range(MAX_HEADERS + 8))
    served = answering(_answer(f"HTTP/1.1 200 OK\nContent-Length: 2\n{lines}\n", b"{}"))

    with pytest.raises(TransportError) as refused:
        reaching(served.base_url)()

    assert refused.value.transient is False
    assert str(MAX_HEADERS) in str(refused.value)


def test_a_scheme_no_transport_reaches_is_refused_before_anything_is_sent(reaching: Any) -> None:
    with pytest.raises(TransportError) as refused:
        reaching("ftp://example.invalid")()

    assert refused.value.transient is False
    assert "ftp" in str(refused.value)


def test_a_url_naming_no_host_is_refused_before_anything_is_sent(reaching: Any) -> None:
    with pytest.raises(TransportError) as refused:
        reaching("http:///only-a-path")()

    assert refused.value.transient is False
    assert "host" in str(refused.value)


def test_a_host_no_name_can_carry_is_configuration_rather_than_weather(reaching: Any) -> None:
    """A URL that cannot be turned into a name is not repeated: the next attempt fails the same way."""
    with pytest.raises(TransportError) as refused:
        reaching(f"http://{AN_UNRESOLVABLE_HOST}")()

    assert refused.value.transient is False


def test_an_api_nothing_is_listening_on_can_be_met_again(reaching: Any) -> None:
    with socket.socket() as held:
        held.bind(("127.0.0.1", 0))
        port = held.getsockname()[1]

    with pytest.raises(TransportError) as refused:
        reaching(f"http://127.0.0.1:{port}")()

    assert refused.value.transient is True


def test_an_answer_that_never_comes_can_be_met_again(answering: Any, reaching: Any) -> None:
    """A server that accepts the connection and writes nothing costs a caller its timeout, not more."""
    served = answering(HOLD)

    with pytest.raises(TransportError) as refused:
        reaching(served.base_url, timeout=0.25)()

    assert refused.value.transient is True


def test_a_refusal_is_an_answer_rather_than_a_failure(answering: Any, reaching: Any) -> None:
    """The status and the body are what say whether repeating could end differently, so they are read."""
    served = answering(
        _answer("HTTP/1.1 429 Too Many Requests\nContent-Length: 15\nRetry-After: 3\n\n", b'{"id": "Rate"}\n')
    )

    status, headers, payload = reaching(served.base_url)()

    assert status == 429
    assert headers["retry-after"] == "3"
    assert payload.startswith(b'{"id"')


def test_every_request_says_which_sdk_at_which_version_is_reaching_the_api(answering: Any, reaching: Any) -> None:
    served = answering(_answer("HTTP/1.1 200 OK\nContent-Length: 2\n\n", b"{}"))

    reaching(served.base_url)()

    sent = served.received[0].decode("utf-8")
    assert "hook0-client-python/" in sent
    assert "Authorization: Bearer token-xyz" in sent
    assert "Hook0-Client-Options: attempts=1," in sent

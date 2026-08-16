"""How a request reaches the API, in the two flavours a Python application comes in.

Both transports answer the same thing — the status and the bytes — and neither of them knows what
the API declares: reading those bytes is the generated half's job, and deciding whether to send
them again is the client's. That is what lets one HTTP implementation serve both the hand-written
event path and every generated method.

Neither transport pulls in a third-party HTTP library. The blocking one is `urllib.request`; the
awaiting one speaks HTTP/1.1 over `asyncio` streams, which is a small protocol to write when a
connection carries exactly one exchange and is closed after it.

Everything a server on the other end controls is bounded here: how long an exchange may take, how
long a status line or a header may be, how many headers may arrive, and how many bytes of body are
read. A server that answers slowly, or forever, costs a caller a timeout rather than a process.
"""

from __future__ import annotations

import asyncio
import http.client
import itertools
import json
import platform
import ssl
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Iterable, Sequence
from typing import Any

from . import __version__

# Longest one attempt at reaching the API is given before it is abandoned.
DEFAULT_REQUEST_TIMEOUT = 10.0

# Largest response body read off a socket, in bytes.
DEFAULT_MAX_RESPONSE_BYTES = 8 * 1024 * 1024

# Longest status line or header line read, in bytes, and most header lines read out of one response.
#
# The head of an answer is written by the other end, so it is bounded like the body: a server that
# is broken or hostile can otherwise spend a caller's memory on headers alone. Both are the numbers
# the conformance corpus names, so that no two SDKs bound different things.
MAX_LINE_BYTES = 64 * 1024
MAX_HEADERS = 64

# Largest whole head read, every line counted together, in bytes.
#
# This is the one that bounds what a head costs, because it bounds the total: a line count and a
# size per line multiply, and the two above admit sixty-four lines of sixty-four kilobytes between
# them. They earn their place by refusing early, on the line that crosses them rather than at the
# end of the head; this one sets the ceiling.
#
# Sixteen kilobytes is the ceiling of the strictest runtime any target runs on, which is what makes
# it a number every target can apply in library code. It is applied here rather than inherited from
# the runtime: `http.client` bounds a line and a count of lines but never a total, and both of its
# numbers are module attributes an application can reassign underneath this package.
MAX_HEAD_BYTES = 16 * 1024

# Most chunks read out of one chunked response body.
MAX_CHUNKS = 4096

# What a JSON request body says it carries.
JSON_MEDIA_TYPE = "application/json"

# Ports the schemes reach when a URL names none.
DEFAULT_PORTS = {"http": 80, "https": 443}

# Longest each part the `User-Agent` is composed of may be, in characters.
#
# The runtime and the operating system are described by the platform rather than by this package, so
# their length is not this package's to guarantee: they are cut here so that the header cannot grow
# with whatever the platform feels like saying. Every part is also stripped of anything the grammar
# of the header uses as punctuation, so a platform cannot forge a shape it does not have.
MAX_USER_AGENT_PART_CHARS = 64


def _clipped(part: str) -> str:
    """One part of the `User-Agent`: printable ASCII, none of the header's own punctuation, cut to size."""
    printable = (character for character in part if " " <= character <= "~" and character not in "();")
    return "".join(itertools.islice(printable, MAX_USER_AGENT_PART_CHARS))


def _user_agent() -> str:
    """Which SDK, at which version, on which runtime and operating system, is talking to the API."""
    runtime = _clipped(f"{platform.python_implementation()} {platform.python_version()}")
    machine = _clipped(f"{platform.system()} {platform.machine()}")
    return f"hook0-client-python/{_clipped(__version__)} ({runtime}; {machine})"


# What every request says it comes from.
#
# Composed once: neither the interpreter nor the machine under it changes while a process runs, and
# an instance can otherwise not tell which SDKs, at which versions, are still reaching it.
USER_AGENT = _user_agent()


def client_options(*, attempts: int, backoff_ms: int, ceiling_ms: int, budget_ms: int) -> str:
    """The retry policy behind a request, as the header every request carries states it.

    The four parts arrive in the order the shared contract fixes and are joined the way
    `X-Hook0-Signature` joins its own: every duration a count of milliseconds and every part an
    integer, so an instance reads the value back by cutting each part at its first `=` and nothing
    here needs a parser of its own. Whole numbers are also what bounds the value without cutting it
    down to a length: four of them are as long as a number written out and no longer, whatever a
    caller configures. Bringing a policy inside the bounds it is honoured under is the caller's,
    since it is the caller that holds the policy.
    """
    return f"attempts={attempts},backoff={backoff_ms},ceiling={ceiling_ms},budget={budget_ms}"


# What a request made through a transport alone says about the retry policy behind it.
#
# A transport reached directly — which is how the generated half of this package is built — issues
# one request and waits for nothing between attempts it does not make. A client that retries hands
# its own policy over instead.
NO_RETRIES = client_options(attempts=1, backoff_ms=0, ceiling_ms=0, budget_ms=0)


class TransportError(Exception):
    """A request that produced no answer to read.

    Several natures of failure land here — a connection refused or reset, an attempt out of time, an
    answer above a ceiling this client set for itself, a URL nothing can be sent to — and only the
    first of them could end differently. Each one therefore says whether it is `transient`, and that
    is what the client retries on: deciding by the type instead would spend four attempts on a
    mistyped API URL and then hand the caller a message that accuses the network.

    A transient failure says nothing about whether the API acted on the request, which is exactly why
    the client sends an event under an identifier it chose itself.
    """

    def __init__(self, detail: str, *, transient: bool) -> None:
        super().__init__(detail)
        self.transient = transient


def _resolved(base_url: str, path: str, query: Sequence[tuple[str, str]]) -> str:
    """Where a request lands: a path of its own replaces the base's, a relative one extends it."""
    resolved = urllib.parse.urljoin(f"{base_url.rstrip('/')}/", path)
    if not query:
        return resolved
    separator = "&" if urllib.parse.urlsplit(resolved).query else "?"
    return f"{resolved}{separator}{urllib.parse.urlencode(list(query))}"


def _encoded(body: Any) -> bytes | None:
    """A request body as the API reads it, or nothing when there is no body to send."""
    if body is None:
        return None
    return json.dumps(body).encode("utf-8")


def _carried(headers: Iterable[tuple[str, str]]) -> dict[str, str]:
    """What an answer carried beside its body, under the names a caller looks them up by.

    A later value wins over an earlier one under the same name, and every name is lowercased, so a
    caller reads a header without knowing which case the server wrote it in.

    This is also where the head is held to its ceilings, for both transports at once: one of them
    reads the head itself and the other is handed it by the standard library, and a bound applied in
    only one of the two is a bound the other does not have.
    """
    read: dict[str, str] = {}
    whole = 0
    for name, value in headers:
        if len(name) + len(value) > MAX_LINE_BYTES:
            raise TransportError(
                f"the API answered a `{name}` header above the {MAX_LINE_BYTES} bytes read at most",
                transient=False,
            )
        whole += len(name) + len(value)
        if whole > MAX_HEAD_BYTES:
            raise TransportError(
                f"the API answered a head above the {MAX_HEAD_BYTES} bytes read at most",
                transient=False,
            )
        read[name.lower()] = value.strip()
        if len(read) > MAX_HEADERS:
            raise TransportError(f"the API answered more than the {MAX_HEADERS} headers read at most", transient=False)
    return read


def _reachable(url: str) -> None:
    """Refuses a URL nothing can be sent to, before anything is sent to it.

    A scheme no transport speaks and a URL naming no host are configuration, not weather: they are
    told apart here so that neither is repeated as if a socket had failed.
    """
    parts = urllib.parse.urlsplit(url)
    if parts.scheme not in DEFAULT_PORTS:
        raise TransportError(f"`{parts.scheme}` is not a scheme this transport reaches", transient=False)
    if not parts.hostname:
        raise TransportError("the API URL names no host", transient=False)


class HttpTransport:
    """Issues one request and waits for the answer."""

    def __init__(
        self,
        base_url: str,
        token: str,
        timeout: float = DEFAULT_REQUEST_TIMEOUT,
        max_response_bytes: int = DEFAULT_MAX_RESPONSE_BYTES,
        client_options: str = NO_RETRIES,
    ) -> None:
        self._base_url = base_url
        self._token = token
        self._timeout = timeout
        self._max_response_bytes = max_response_bytes
        self._client_options = client_options

    def request(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]],
        body: Any = None,
    ) -> tuple[int, bytes]:
        """What the API answered, whether or not it answered a success.

        This is the shape the generated half of this package reads, which is the status and the
        bytes. A caller that also needs what the answer carried beside its body — the delay a paced
        instance names is one — asks `deliver` for it.
        """
        status, _, payload = self.deliver(method, path, query, body)
        return status, payload

    def deliver(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]],
        body: Any = None,
    ) -> tuple[int, dict[str, str], bytes]:
        """What the API answered, headers included, whether or not it answered a success."""
        url = _resolved(self._base_url, path, query)
        _reachable(url)

        data = _encoded(body)
        request = urllib.request.Request(url, data=data, method=method)
        request.add_header("Authorization", f"Bearer {self._token}")
        request.add_header("Accept", JSON_MEDIA_TYPE)
        request.add_header("User-Agent", USER_AGENT)
        request.add_header("Hook0-Client-Options", self._client_options)
        if data is not None:
            request.add_header("Content-Type", JSON_MEDIA_TYPE)

        try:
            with urllib.request.urlopen(request, timeout=self._timeout) as answer:  # noqa: S310
                return answer.status, _carried(answer.headers.items()), self._read(answer)
        except urllib.error.HTTPError as refused:
            # A refusal is an answer: the status and the body are what say whether repeating the
            # request could end differently, so they are read rather than raised over.
            with refused:
                return refused.code, _carried(refused.headers.items()), self._read(refused)
        except http.client.IncompleteRead as truncated:
            # A body that stopped mid-way is the one protocol failure worth meeting again: the next
            # answer could carry the whole of it.
            raise TransportError(str(truncated), transient=True) from truncated
        except http.client.HTTPException as unreadable:
            # A head this client could not read whole — a line above what the standard library
            # holds, more headers than it counts — reads the same way the second time.
            raise TransportError(str(unreadable), transient=False) from unreadable
        except OSError as unreachable:
            raise TransportError(str(unreachable), transient=True) from unreachable
        except ValueError as unusable:
            raise TransportError(str(unusable), transient=False) from unusable

    def _read(self, answer: Any) -> bytes:
        """The body of an answer, up to what this transport agrees to hold."""
        payload = answer.read(self._max_response_bytes + 1)
        if len(payload) > self._max_response_bytes:
            raise TransportError(
                f"the API answered more than the {self._max_response_bytes} bytes read at most",
                transient=False,
            )
        return payload


class AsyncHttpTransport:
    """Issues one request and awaits the answer."""

    def __init__(
        self,
        base_url: str,
        token: str,
        timeout: float = DEFAULT_REQUEST_TIMEOUT,
        max_response_bytes: int = DEFAULT_MAX_RESPONSE_BYTES,
        client_options: str = NO_RETRIES,
    ) -> None:
        self._base_url = base_url
        self._token = token
        self._timeout = timeout
        self._max_response_bytes = max_response_bytes
        self._client_options = client_options

    async def request(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]],
        body: Any = None,
    ) -> tuple[int, bytes]:
        """What the API answered, whether or not it answered a success.

        This is the shape the generated half of this package reads; `deliver` is the same exchange
        with what the answer carried beside its body.
        """
        status, _, payload = await self.deliver(method, path, query, body)
        return status, payload

    async def deliver(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]],
        body: Any = None,
    ) -> tuple[int, dict[str, str], bytes]:
        """What the API answered, headers included, whether or not it answered a success."""
        url = _resolved(self._base_url, path, query)
        _reachable(url)

        try:
            return await asyncio.wait_for(self._exchange(url, method, _encoded(body)), self._timeout)
        except TimeoutError as expired:
            raise TransportError(f"the API did not answer within {self._timeout}s", transient=True) from expired
        except (OSError, ssl.SSLError, EOFError) as unreachable:
            raise TransportError(str(unreachable), transient=True) from unreachable
        except ValueError as unusable:
            raise TransportError(str(unusable), transient=False) from unusable

    async def _exchange(self, url: str, method: str, data: bytes | None) -> tuple[int, dict[str, str], bytes]:
        """One request written to a connection, and the answer read back off it."""
        parts = urllib.parse.urlsplit(url)
        port = parts.port if parts.port else DEFAULT_PORTS[parts.scheme]
        context = ssl.create_default_context() if parts.scheme == "https" else None
        reader, writer = await asyncio.open_connection(
            parts.hostname,
            port,
            ssl=context,
            limit=MAX_LINE_BYTES,
        )

        try:
            writer.write(self._request_bytes(parts, method, data))
            await writer.drain()
            return await self._answer(reader)
        finally:
            writer.close()
            # A connection that will not close is not worth waiting on: the exchange is over, and
            # the timeout around it has better things to bound.
            try:
                await writer.wait_closed()
            except (OSError, ssl.SSLError):
                pass

    def _request_bytes(self, parts: urllib.parse.SplitResult, method: str, data: bytes | None) -> bytes:
        """One request, as the wire carries it."""
        target = parts.path if parts.path else "/"
        if parts.query:
            target = f"{target}?{parts.query}"

        lines = [
            f"{method} {target} HTTP/1.1",
            f"Host: {parts.netloc}",
            f"Authorization: Bearer {self._token}",
            f"Accept: {JSON_MEDIA_TYPE}",
            f"User-Agent: {USER_AGENT}",
            f"Hook0-Client-Options: {self._client_options}",
            # One exchange per connection: nothing here pools them, and a server that closes is
            # what makes a body of unstated length readable to its end.
            "Connection: close",
        ]
        if data is not None:
            lines.append(f"Content-Type: {JSON_MEDIA_TYPE}")
            lines.append(f"Content-Length: {len(data)}")

        head = ("\r\n".join(lines) + "\r\n\r\n").encode("utf-8")
        if data is None:
            return head
        return head + data

    async def _answer(self, reader: asyncio.StreamReader) -> tuple[int, dict[str, str], bytes]:
        """The status, the headers and the body of one answer."""
        status = _status(await self._line(reader))

        carried: list[tuple[str, str]] = []
        length: int | None = None
        chunked = False
        for _ in range(MAX_HEADERS):
            line = (await self._line(reader)).strip()
            if not line:
                break
            name, _, value = line.partition(":")
            name = name.strip().lower()
            value = value.strip()
            carried.append((name, value))
            if name == "content-length":
                length = _length(value)
            elif name == "transfer-encoding":
                chunked = "chunked" in value.lower()
        else:
            raise TransportError(f"the API answered more than the {MAX_HEADERS} headers read at most", transient=False)

        headers = _carried(carried)
        if chunked:
            return status, headers, await self._chunked(reader)
        if length is not None:
            if length > self._max_response_bytes:
                raise TransportError(
                    f"the API announced {length} bytes, above the {self._max_response_bytes} read",
                    transient=False,
                )
            return status, headers, await reader.readexactly(length)

        # No length and no chunking: the body runs to the end of the connection, which the request
        # asked the server to close.
        payload = await reader.read(self._max_response_bytes + 1)
        if len(payload) > self._max_response_bytes:
            raise TransportError(
                f"the API answered more than the {self._max_response_bytes} bytes read at most",
                transient=False,
            )
        return status, headers, payload

    async def _line(self, reader: asyncio.StreamReader) -> str:
        """One line of the head of an answer, bounded by what a line may hold."""
        try:
            line = await reader.readuntil(b"\n")
        except asyncio.LimitOverrunError as overrun:
            raise TransportError(
                f"the API answered a line above the {MAX_LINE_BYTES} bytes read", transient=False
            ) from overrun
        except asyncio.IncompleteReadError as truncated:
            raise TransportError("the API closed the connection mid-answer", transient=True) from truncated
        return line.decode("utf-8", errors="replace")

    async def _chunked(self, reader: asyncio.StreamReader) -> bytes:
        """A body written as a series of sized chunks, read up to what this transport holds."""
        payload = bytearray()
        for _ in range(MAX_CHUNKS):
            header = (await self._line(reader)).strip()
            size = _chunk_size(header)
            if size == 0:
                return bytes(payload)
            if len(payload) + size > self._max_response_bytes:
                raise TransportError(
                    f"the API answered more than the {self._max_response_bytes} bytes read at most",
                    transient=False,
                )
            payload.extend(await reader.readexactly(size))
            await reader.readexactly(2)
        raise TransportError(f"the API answered more than the {MAX_CHUNKS} chunks read at most", transient=False)


def _status(line: str) -> int:
    """The status of an answer, out of the line that opens it."""
    fields = line.split(" ", 2)
    if len(fields) < 2 or not fields[0].startswith("HTTP/"):
        raise TransportError(f"the API did not answer with a status line: {line.strip()!r}", transient=False)
    try:
        return int(fields[1])
    except ValueError as unreadable:
        raise TransportError(f"`{fields[1]}` is not a status", transient=False) from unreadable


def _length(value: str) -> int:
    """How long an answer says its body is."""
    try:
        length = int(value)
    except ValueError as unreadable:
        raise TransportError(f"`{value}` is not a length", transient=False) from unreadable
    if length < 0:
        raise TransportError(f"`{value}` is not a length", transient=False)
    return length


def _chunk_size(header: str) -> int:
    """How long one chunk of an answer is, out of the line that opens it."""
    size, _, _ = header.partition(";")
    try:
        return int(size, 16)
    except ValueError as unreadable:
        raise TransportError(f"`{header}` is not a chunk size", transient=False) from unreadable

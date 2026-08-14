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
import json
import ssl
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Sequence
from typing import Any

# Longest one attempt at reaching the API is given before it is abandoned.
DEFAULT_REQUEST_TIMEOUT = 10.0

# Largest response body read off a socket, in bytes.
DEFAULT_MAX_RESPONSE_BYTES = 8 * 1024 * 1024

# Longest status line or header line read, in bytes.
MAX_LINE_BYTES = 64 * 1024

# Most header lines read out of one response.
MAX_HEADERS = 128

# Most chunks read out of one chunked response body.
MAX_CHUNKS = 4096

# What a JSON request body says it carries.
JSON_MEDIA_TYPE = "application/json"

# Ports the schemes reach when a URL names none.
DEFAULT_PORTS = {"http": 80, "https": 443}


class TransportError(Exception):
    """A request that got no answer: a connection refused or reset, or an attempt out of time.

    None of these says whether the API acted on the request, which is exactly why the client sends
    an event under an identifier it chose itself.
    """


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


class HttpTransport:
    """Issues one request and waits for the answer."""

    def __init__(
        self,
        base_url: str,
        token: str,
        timeout: float = DEFAULT_REQUEST_TIMEOUT,
        max_response_bytes: int = DEFAULT_MAX_RESPONSE_BYTES,
    ) -> None:
        self._base_url = base_url
        self._token = token
        self._timeout = timeout
        self._max_response_bytes = max_response_bytes

    def request(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]],
        body: Any = None,
    ) -> tuple[int, bytes]:
        """What the API answered, whether or not it answered a success."""
        data = _encoded(body)
        request = urllib.request.Request(_resolved(self._base_url, path, query), data=data, method=method)
        request.add_header("Authorization", f"Bearer {self._token}")
        request.add_header("Accept", JSON_MEDIA_TYPE)
        if data is not None:
            request.add_header("Content-Type", JSON_MEDIA_TYPE)

        try:
            with urllib.request.urlopen(request, timeout=self._timeout) as answer:  # noqa: S310
                return answer.status, self._read(answer)
        except urllib.error.HTTPError as refused:
            # A refusal is an answer: the status and the body are what say whether repeating the
            # request could end differently, so they are read rather than raised over.
            with refused:
                return refused.code, self._read(refused)
        except (OSError, http.client.HTTPException, ValueError) as unreachable:
            raise TransportError(str(unreachable)) from unreachable

    def _read(self, answer: Any) -> bytes:
        """The body of an answer, up to what this transport agrees to hold."""
        payload = answer.read(self._max_response_bytes + 1)
        if len(payload) > self._max_response_bytes:
            raise TransportError(f"the API answered more than the {self._max_response_bytes} bytes read at most")
        return payload


class AsyncHttpTransport:
    """Issues one request and awaits the answer."""

    def __init__(
        self,
        base_url: str,
        token: str,
        timeout: float = DEFAULT_REQUEST_TIMEOUT,
        max_response_bytes: int = DEFAULT_MAX_RESPONSE_BYTES,
    ) -> None:
        self._base_url = base_url
        self._token = token
        self._timeout = timeout
        self._max_response_bytes = max_response_bytes

    async def request(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]],
        body: Any = None,
    ) -> tuple[int, bytes]:
        """What the API answered, whether or not it answered a success."""
        url = _resolved(self._base_url, path, query)
        try:
            return await asyncio.wait_for(self._exchange(url, method, _encoded(body)), self._timeout)
        except TimeoutError as expired:
            raise TransportError(f"the API did not answer within {self._timeout}s") from expired
        except (OSError, ssl.SSLError, EOFError, ValueError) as unreachable:
            raise TransportError(str(unreachable)) from unreachable

    async def _exchange(self, url: str, method: str, data: bytes | None) -> tuple[int, bytes]:
        """One request written to a connection, and the answer read back off it."""
        parts = urllib.parse.urlsplit(url)
        if parts.scheme not in DEFAULT_PORTS:
            raise TransportError(f"`{parts.scheme}` is not a scheme this transport reaches")
        if not parts.hostname:
            raise TransportError("the API URL names no host")

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

    async def _answer(self, reader: asyncio.StreamReader) -> tuple[int, bytes]:
        """The status and the body of one answer."""
        status = _status(await self._line(reader))

        length: int | None = None
        chunked = False
        for _ in range(MAX_HEADERS):
            line = (await self._line(reader)).strip()
            if not line:
                break
            name, _, value = line.partition(":")
            name = name.strip().lower()
            value = value.strip()
            if name == "content-length":
                length = _length(value)
            elif name == "transfer-encoding":
                chunked = "chunked" in value.lower()
        else:
            raise TransportError(f"the API answered more than the {MAX_HEADERS} headers read at most")

        if chunked:
            return status, await self._chunked(reader)
        if length is not None:
            if length > self._max_response_bytes:
                raise TransportError(f"the API announced {length} bytes, above the {self._max_response_bytes} read")
            return status, await reader.readexactly(length)

        # No length and no chunking: the body runs to the end of the connection, which the request
        # asked the server to close.
        payload = await reader.read(self._max_response_bytes + 1)
        if len(payload) > self._max_response_bytes:
            raise TransportError(f"the API answered more than the {self._max_response_bytes} bytes read at most")
        return status, payload

    async def _line(self, reader: asyncio.StreamReader) -> str:
        """One line of the head of an answer, bounded by what a line may hold."""
        try:
            line = await reader.readuntil(b"\n")
        except asyncio.LimitOverrunError as overrun:
            raise TransportError(f"the API answered a line above the {MAX_LINE_BYTES} bytes read") from overrun
        except asyncio.IncompleteReadError as truncated:
            raise TransportError("the API closed the connection mid-answer") from truncated
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
                raise TransportError(f"the API answered more than the {self._max_response_bytes} bytes read at most")
            payload.extend(await reader.readexactly(size))
            await reader.readexactly(2)
        raise TransportError(f"the API answered more than the {MAX_CHUNKS} chunks read at most")


def _status(line: str) -> int:
    """The status of an answer, out of the line that opens it."""
    fields = line.split(" ", 2)
    if len(fields) < 2 or not fields[0].startswith("HTTP/"):
        raise TransportError(f"the API did not answer with a status line: {line.strip()!r}")
    try:
        return int(fields[1])
    except ValueError as unreadable:
        raise TransportError(f"`{fields[1]}` is not a status") from unreadable


def _length(value: str) -> int:
    """How long an answer says its body is."""
    try:
        length = int(value)
    except ValueError as unreadable:
        raise TransportError(f"`{value}` is not a length") from unreadable
    if length < 0:
        raise TransportError(f"`{value}` is not a length")
    return length


def _chunk_size(header: str) -> int:
    """How long one chunk of an answer is, out of the line that opens it."""
    size, _, _ = header.partition(";")
    try:
        return int(size, 16)
    except ValueError as unreadable:
        raise TransportError(f"`{header}` is not a chunk size") from unreadable

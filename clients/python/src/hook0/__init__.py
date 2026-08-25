"""The Python SDK for Hook0.

Two halves live here. This one is hand-written: sending an event, upserting the event types an
application uses, and verifying that a webhook came from Hook0 unchanged. The other is generated
from the OpenAPI snapshot the API commits — one class per schema it declares, one exception per
problem it can report, one method per operation — and is reached through `hook0.generated`, over
either transport this module exports.
"""

from __future__ import annotations

# The version this package is published under, and the one it reports on the wire. It is bound
# before anything below is imported because the transport reads it from here to compose the header
# every request carries; the conformance suite reads `pyproject.toml` and holds that header against
# it, so a number bumped in one place and not the other fails a case rather than shipping.
__version__ = "2.0.1"

from .client import (
    ALREADY_INGESTED,
    DEFAULT_MAX_PAYLOAD_BYTES,
    Event,
    EventType,
    Hook0AsyncClient,
    Hook0Client,
    Hook0ClientOptions,
    MAX_ATTEMPTS_CAP,
    RetryPolicy,
    generate_event_id,
)
from .errors import Hook0ClientError
from .runtime import DecodeError
from .signature import (
    Signature,
    verify_webhook_signature,
    verify_webhook_signature_with_current_time,
)
from .transport import (
    AsyncHttpTransport,
    DEFAULT_MAX_RESPONSE_BYTES,
    DEFAULT_REQUEST_TIMEOUT,
    HttpTransport,
    TransportError,
)

__all__ = [
    "ALREADY_INGESTED",
    "DEFAULT_MAX_PAYLOAD_BYTES",
    "DEFAULT_MAX_RESPONSE_BYTES",
    "DEFAULT_REQUEST_TIMEOUT",
    "MAX_ATTEMPTS_CAP",
    "AsyncHttpTransport",
    "DecodeError",
    "Event",
    "EventType",
    "Hook0AsyncClient",
    "Hook0Client",
    "Hook0ClientError",
    "Hook0ClientOptions",
    "HttpTransport",
    "RetryPolicy",
    "Signature",
    "TransportError",
    "generate_event_id",
    "verify_webhook_signature",
    "verify_webhook_signature_with_current_time",
]

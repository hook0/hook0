# The rest of the file, for every Python example of the SDK reference.
#
# A snippet on a page is written for a reader: it leaves out the imports it already showed higher up
# the page, it assumes a client or an event is already built, and it names an application id, a
# token or a secret without saying where it came from. Each region below is the file that snippet
# would live in, with a hole where it goes. The page points at one by name on the fence, so what a
# snippet is standing on is one word away from the snippet itself.
#
# This file never runs as it stands, and mypy never sees it whole: each region becomes the one file
# of its own example, under a directory named after it, which is why a region that needs a name in
# scope declares it above the hole rather than assuming the file around it.

# HARNESS send
application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
token = "a-service-token"

EXAMPLE

# END HARNESS

# HARNESS event
from datetime import datetime, timezone

from hook0 import Event

# The value the page shows, held so that every field of it is checked against the client.
event = (
    EXAMPLE
)

# END HARNESS

# HARNESS async
application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
token = "a-service-token"


# `await` is refused outside a function body, which a page showing it bare never has to say.
async def send() -> None:
    EXAMPLE


# END HARNESS

# HARNESS options
application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
token = "a-service-token"

EXAMPLE

# END HARNESS

# HARNESS verify
subscription_secret = "a-subscription-secret"


# What the page calls `request`: a stand-in typed the way a web framework's own request carries a
# signature through, so the snippet is checked against the same shapes `verify_webhook_signature`
# declares rather than against nothing.
class _Request:
    headers: dict[str, str]
    body: bytes


request = _Request()
request.headers = {"X-Hook0-Signature": "t=1700000000,v1=deadbeef"}
request.body = b'{"invoice": "in_123"}'

EXAMPLE

# END HARNESS

# HARNESS flask
subscription_secret = "a-subscription-secret"

EXAMPLE

# END HARNESS

# HARNESS upsert
from hook0 import Hook0Client

application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
token = "a-service-token"
client = Hook0Client("https://app.hook0.com/api/v1", application_id, token)

EXAMPLE

# END HARNESS

# HARNESS parse
EXAMPLE

# END HARNESS

# HARNESS rest
token = "a-service-token"
application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"

EXAMPLE

# END HARNESS

# HARNESS errors
import logging

from hook0 import Event, Hook0Client

logger = logging.getLogger(__name__)
client = Hook0Client("https://app.hook0.com/api/v1", "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21", "a-service-token")
event = Event(
    event_type="billing.invoice.paid",
    payload='{"invoice": "in_123"}',
    payload_content_type="application/json",
)

EXAMPLE

# END HARNESS

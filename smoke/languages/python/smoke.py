"""The Python client against a Hook0 that is really running.

Three things the loopback suite cannot ask: whether an application secret the API minted is
accepted, whether a second send under an identifier already ingested is reported as the conflict it
is, and whether a signature the output worker computed verifies. Everything else about this client
is settled by `clients/python/tests`.
"""

import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent.parent.parent / "clients" / "python" / "src"))

from hook0 import Event, Hook0Client, Hook0ClientError, verify_webhook_signature  # noqa: E402

# The conflict the API answers a duplicated ingestion with.
ALREADY_INGESTED = "EventAlreadyIngested"


def setting(name: str) -> str:
    """A setting the harness passes, or a refusal naming it."""
    value = os.environ.get(name, "")
    if not value:
        raise SystemExit(f"{name} is not set")
    return value


def event(event_type: str, event_id: str | None) -> Event:
    """The event both sends carry, under the identifier the caller names."""
    return Event(
        event_type=event_type,
        payload='{"from":"the python smoke"}',
        payload_content_type="application/json",
        labels={"language": "python"},
        event_id=event_id,
    )


def verify(delivery: Path) -> None:
    """Verifies what the output worker really delivered, with this client's own verification."""
    headers = {}
    for line in (delivery / "headers").read_text().splitlines():
        name, separator, value = line.partition(": ")
        if separator:
            headers[name] = value

    verify_webhook_signature(
        (delivery / "signature").read_text().strip(),
        (delivery / "body").read_bytes(),
        headers,
        (delivery / "secret").read_text().strip(),
        int((delivery / "tolerance").read_text().strip()),
    )


def main() -> None:
    client = Hook0Client(
        setting("HOOK0_API_URL"),
        setting("HOOK0_APPLICATION_ID"),
        setting("HOOK0_TOKEN"),
    )
    event_type = setting("HOOK0_EVENT_TYPE")

    sent = client.send_event(event(event_type, None))
    print(f"ingested {sent}")

    try:
        client.send_event(event(event_type, sent))
    except Hook0ClientError as refused:
        said = str(refused)
        if ALREADY_INGESTED not in said:
            raise SystemExit(
                f"the second send failed without naming {ALREADY_INGESTED}: {said}"
            ) from refused
        print(f"the second send reported {ALREADY_INGESTED}")
    else:
        raise SystemExit("sending the same event twice was accepted twice")

    verify(Path(setting("HOOK0_DELIVERY")))
    print("the signature the instance produced verifies")


main()

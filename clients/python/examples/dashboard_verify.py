"""What the dashboard shows under "Verify a webhook", for Python.

Sending is only half of what a reader has come to do, and it is the easier half. This is the one the
SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside the send
rather than leaving it to be found later.

The secret is read from the environment on purpose. The dashboard cannot know which subscription a
reader means — outside the onboarding it loads none, and an application may have several — so it
points at the subscription instead of guessing one, and no second secret is put on screen.

Read the markers as in `dashboard_send.py`: `hook0:snippet` is what is displayed, everything outside
it is what holds this file against the client.
"""

# hook0:snippet:begin
import os

from hook0 import Hook0ClientError, verify_webhook_signature


# Verify against the *raw* body: one that has been parsed and serialised again no longer hashes to
# what was signed. The tolerance is bilateral, so a delivery dated too far ahead is refused exactly
# like one dated too far behind.
def accept(signature: str, body: bytes, headers: dict[str, str]) -> bool:
    # The secret of the subscription being verified, which the dashboard links to rather than
    # prints: it cannot know which subscription a reader means, and an application may have several.
    # A variable nobody exported and one exported empty are the same defect and are raised on
    # together: a secret that is empty hashes every genuine delivery to the wrong code, so each one
    # comes back refused as forged and the variable is the last place anyone thinks to look.
    secret = os.environ.get("HOOK0_SUBSCRIPTION_SECRET")
    if not secret:
        raise RuntimeError("HOOK0_SUBSCRIPTION_SECRET is not set")
    try:
        verify_webhook_signature(signature, body, headers, secret, 300)
    except Hook0ClientError:
        return False
    return True


# hook0:snippet:end


if __name__ == "__main__":
    # Nothing here is ever run by the pipeline: this file exists to be held against the real client.
    accepted = accept("", b"", {"x-hook0-signature": ""})
    print(f"accepted: {accepted}")

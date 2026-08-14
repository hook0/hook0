"""What a webhook has to carry to be accepted, and every way it can fail to.

The signatures below are produced the way Hook0 produces them — an HMAC-SHA256 over the moment, the
covered headers and the body — rather than read back from the module under test, so a case fails
when the module changes what it computes over.
"""

from __future__ import annotations

import datetime
import hashlib
import hmac

import pytest
from conftest import TEST_TIMEOUT

from hook0 import Hook0ClientError, verify_webhook_signature, verify_webhook_signature_with_current_time

pytestmark = pytest.mark.timeout(TEST_TIMEOUT)

SECRET = "a-subscription-secret"
PAYLOAD = b'{"hello": "world"}'
NOW = datetime.datetime(2026, 8, 14, 12, 0, 0, tzinfo=datetime.UTC)
TOLERANCE = 300.0


def signed_at(moment: datetime.datetime) -> int:
    return int(moment.timestamp())


def body_code(timestamp: int, payload: bytes = PAYLOAD, secret: str = SECRET) -> str:
    """The `v0` code: the moment, then the body."""
    code = hmac.new(secret.encode("utf-8"), digestmod=hashlib.sha256)
    code.update(f"{timestamp}.".encode())
    code.update(payload)
    return code.hexdigest()


def headers_code(
    timestamp: int,
    covered: list[tuple[str, str]],
    payload: bytes = PAYLOAD,
    secret: str = SECRET,
) -> str:
    """The `v1` code: the moment, the covered names, their values, then the body."""
    code = hmac.new(secret.encode("utf-8"), digestmod=hashlib.sha256)
    code.update(f"{timestamp}.".encode())
    code.update(" ".join(name for name, _ in covered).encode("utf-8"))
    code.update(b".")
    code.update(".".join(value for _, value in covered).encode("utf-8"))
    code.update(b".")
    code.update(payload)
    return code.hexdigest()


def verify(signature: str, headers: dict[str, str], current_time: datetime.datetime = NOW) -> bool:
    return verify_webhook_signature_with_current_time(signature, PAYLOAD, headers, SECRET, TOLERANCE, current_time)


def test_a_v0_signature_over_the_body_is_accepted() -> None:
    timestamp = signed_at(NOW)

    assert verify(f"t={timestamp},v0={body_code(timestamp)}", {})


def test_a_v1_signature_over_the_covered_headers_is_accepted() -> None:
    timestamp = signed_at(NOW)
    covered = [("x-event-id", "abc"), ("x-event-type", "auth.user.create")]
    signature = f"t={timestamp},h={' '.join(name for name, _ in covered)},v1={headers_code(timestamp, covered)}"

    assert verify(signature, dict(covered))


def test_the_covered_headers_are_read_in_the_order_the_signature_lists_them() -> None:
    timestamp = signed_at(NOW)
    covered = [("x-event-id", "abc"), ("x-event-type", "auth.user.create")]
    swapped = list(reversed(covered))
    # The values are signed in the order `h` lists the names, so the same two headers signed the
    # other way round is a different message and must not verify.
    signature = f"t={timestamp},h={' '.join(name for name, _ in covered)},v1={headers_code(timestamp, swapped)}"

    with pytest.raises(Hook0ClientError):
        verify(signature, dict(covered))


def test_the_delivered_headers_are_found_whatever_case_they_arrived_in() -> None:
    timestamp = signed_at(NOW)
    covered = [("x-event-id", "abc")]
    signature = f"t={timestamp},h=x-event-id,v1={headers_code(timestamp, covered)}"

    assert verify(signature, {"X-Event-Id": "abc"})


def test_a_v1_signature_wins_over_a_v0_one_that_would_have_verified() -> None:
    timestamp = signed_at(NOW)
    covered = [("x-event-id", "abc")]
    # A sender offering both must not have the weaker of the two accepted on the strength of the
    # stronger one being wrong.
    signature = (
        f"t={timestamp},h=x-event-id,v0={body_code(timestamp)},v1={headers_code(timestamp, [('x-event-id', 'other')])}"
    )

    with pytest.raises(Hook0ClientError):
        verify(signature, dict(covered))


def test_a_header_the_signature_covers_but_the_request_did_not_carry_is_refused() -> None:
    timestamp = signed_at(NOW)
    covered = [("x-event-id", "abc")]
    signature = f"t={timestamp},h=x-event-id,v1={headers_code(timestamp, covered)}"

    with pytest.raises(Hook0ClientError) as refused:
        verify(signature, {})

    assert "was not delivered" in str(refused.value)


def test_a_code_that_is_not_whole_hexadecimal_is_refused() -> None:
    timestamp = signed_at(NOW)
    # A decoder that stops at the first bad character compares a prefix of the right code, which
    # is a signature anyone can produce.
    truncated = body_code(timestamp)[:20] + "zz" + body_code(timestamp)[22:]

    with pytest.raises(Hook0ClientError) as refused:
        verify(f"t={timestamp},v0={truncated}", {})

    assert "not hexadecimal" in str(refused.value)


def test_a_code_of_odd_length_is_refused() -> None:
    timestamp = signed_at(NOW)

    with pytest.raises(Hook0ClientError):
        verify(f"t={timestamp},v0={body_code(timestamp)[:-1]}", {})


def test_a_signature_signed_too_long_ago_is_refused() -> None:
    stale = NOW - datetime.timedelta(seconds=TOLERANCE + 1)
    timestamp = signed_at(stale)

    with pytest.raises(Hook0ClientError) as refused:
        verify(f"t={timestamp},v0={body_code(timestamp)}", {})

    assert "outside the" in str(refused.value)


def test_a_signature_signed_too_far_in_the_future_is_refused() -> None:
    ahead = NOW + datetime.timedelta(seconds=TOLERANCE + 1)
    timestamp = signed_at(ahead)

    # A window that only looks backwards is one a forged timestamp can widen without limit.
    with pytest.raises(Hook0ClientError) as refused:
        verify(f"t={timestamp},v0={body_code(timestamp)}", {})

    assert "outside the" in str(refused.value)


def test_a_signature_at_the_edge_of_the_window_is_accepted_on_both_sides() -> None:
    for edge in (NOW - datetime.timedelta(seconds=TOLERANCE), NOW + datetime.timedelta(seconds=TOLERANCE)):
        timestamp = signed_at(edge)
        assert verify(f"t={timestamp},v0={body_code(timestamp)}", {})


def test_a_signature_carrying_no_moment_is_refused() -> None:
    with pytest.raises(Hook0ClientError) as refused:
        verify(f"v0={body_code(0)},h=", {})

    assert "`t`" in str(refused.value)


def test_a_signature_carrying_no_code_is_refused() -> None:
    timestamp = signed_at(NOW)

    with pytest.raises(Hook0ClientError):
        verify(f"t={timestamp},h=x-event-id", {"x-event-id": "abc"})


def test_a_body_that_changed_after_it_was_signed_is_refused() -> None:
    timestamp = signed_at(NOW)
    signature = f"t={timestamp},v0={body_code(timestamp, payload=b'something else')}"

    with pytest.raises(Hook0ClientError) as refused:
        verify(signature, {})

    assert "does not match" in str(refused.value)


def test_a_signature_made_under_another_secret_is_refused() -> None:
    timestamp = signed_at(NOW)
    signature = f"t={timestamp},v0={body_code(timestamp, secret='another-secret')}"

    with pytest.raises(Hook0ClientError):
        verify(signature, {})


def test_the_parts_of_a_signature_are_read_around_the_spaces_they_arrived_with() -> None:
    timestamp = signed_at(NOW)

    assert verify(f" t = {timestamp} , v0 = {body_code(timestamp)} ", {})


def test_verifying_against_the_current_moment_accepts_a_signature_made_now() -> None:
    timestamp = signed_at(datetime.datetime.now(datetime.UTC))

    assert verify_webhook_signature(
        f"t={timestamp},v0={body_code(timestamp)}",
        PAYLOAD,
        {},
        SECRET,
        TOLERANCE,
    )


def test_headers_given_as_pairs_are_read_like_headers_given_as_a_mapping() -> None:
    timestamp = signed_at(NOW)
    covered = [("x-event-id", "abc")]
    signature = f"t={timestamp},h=x-event-id,v1={headers_code(timestamp, covered)}"

    assert verify(signature, dict(covered))
    assert verify_webhook_signature_with_current_time(signature, PAYLOAD, covered, SECRET, TOLERANCE, NOW)

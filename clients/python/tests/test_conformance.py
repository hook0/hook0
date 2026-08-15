"""The cases the shared conformance corpus dictates, run against this client.

The corpus sits at `clients/conformance`, is hand-authored, and is read by the suite of every SDK.
Nothing below writes down a verdict, a bound or a signature of its own: they are read out of the
committed documents and this client is driven against them over a real socket, in both of the
flavours it comes in. A case added to the corpus is therefore exercised here without this file being
touched, and a verdict changed there fails here until this client agrees with it again.
"""

from __future__ import annotations

import datetime
import json
import time
import tomllib
from pathlib import Path
from typing import Any

import pytest
from conftest import Caller, FakeHook0Api, ScriptedResponse, TEST_TIMEOUT, an_event, ingested, prompt_options

from hook0 import (
    Hook0Client,
    Hook0ClientError,
    Hook0ClientOptions,
    RetryPolicy,
    verify_webhook_signature_with_current_time,
)

pytestmark = pytest.mark.timeout(TEST_TIMEOUT)

# Where the shared contract sits, from the directory this suite runs out of.
CORPUS = Path(__file__).resolve().parents[2] / "conformance"

# Largest document of the corpus read back. The corpus is committed, so one above this is one that
# grew out of shape rather than one somebody meant.
MAX_CORPUS_BYTES = 512 * 1024

INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac"

# The schedule a case that is not about waiting spends between attempts, in seconds.
PROMPT_BACKOFF = 0.005

# The budget the delay cases share. A delay the API names above it is expected to be cut down to it,
# so this also bounds what those cases cost.
DELAY_BUDGET = 1.1

# What a wait may overshoot by before it is read as more than what was asked for: a loopback round
# trip, a timer and a scheduler all sit inside it.
DELAY_SLACK = 0.6


def document(name: str) -> Any:
    """One document of the shared contract, bounded before it is parsed."""
    path = CORPUS / name
    size = path.stat().st_size
    if size > MAX_CORPUS_BYTES:
        raise AssertionError(f"{path} is {size} bytes long, above the {MAX_CORPUS_BYTES} read back")
    return json.loads(path.read_text(encoding="utf-8"))


def manifest_version() -> str:
    """The version this package is published under, which is the one it reports on the wire.

    Read out of the manifest rather than written down again here, so that the number the client
    carries and the number the package ships as cannot disagree without this suite saying so.
    """
    manifest = Path(__file__).resolve().parents[1] / "pyproject.toml"
    return str(tomllib.loads(manifest.read_text(encoding="utf-8"))["project"]["version"])


def template_chunks(template: str, bound: dict[str, str]) -> list[str]:
    """What a value of the request document is made of, once the holes this suite can fill are.

    A value is a template: `${name}` is a hole and everything around it is literal. A hole named in
    `bound` becomes part of the literal text around it; one that is not is a hole no suite can fill
    without reimplementing the client it is testing, and it separates two chunks. A template whose
    holes are all bound is therefore one chunk, and the whole value is that chunk.
    """
    chunks = [""]
    rest = template

    while (opened := rest.find("${")) >= 0:
        closed = rest.find("}", opened)
        if closed < 0:
            break
        chunks[-1] += rest[:opened]

        name = rest[opened + 2 : closed]
        if name in bound:
            chunks[-1] += bound[name]
        else:
            chunks.append("")
        rest = rest[closed + 1 :]

    chunks[-1] += rest
    return chunks


def matches_chunks(chunks: list[str], carried: str) -> bool:
    """Whether what arrived is what those chunks describe.

    The literal text in order, anchored at both ends, with something non-empty standing in every
    hole between them.
    """
    if len(chunks) == 1:
        return carried == chunks[0]
    if not carried.startswith(chunks[0]):
        return False

    rest = carried[len(chunks[0]) :]
    for chunk in chunks[1:-1]:
        # A hole stands before this chunk, and nothing is not something, so the search starts past
        # whatever fills it.
        found = rest[1:].find(chunk)
        if found < 0:
            return False
        rest = rest[1 + found + len(chunk) :]

    last = chunks[-1]
    return len(rest) > len(last) and rest.endswith(last)


RETRY = document("retry.json")
BOUNDS = document("bounds.json")["bounds"]
SIGNATURE = document("signature.json")
REQUEST = document("request.json")
VERSION = manifest_version()

# How a refusal the corpus names reads in this client's own words. Every name the corpus declares is
# looked up here, so one added there stops this suite until it is mapped rather than passing under
# whatever the client happened to say.
REFUSALS = {
    "code_not_hexadecimal": "not hexadecimal",
    "header_not_delivered": "was not delivered",
    "code_mismatch": "does not match",
    "outside_tolerance": "outside the",
}


def answered(status: int, problem: str, headers: dict[str, str] | None = None) -> ScriptedResponse:
    """What the API says when it refuses a request, in the shape every Hook0 failure takes."""
    return ScriptedResponse(
        status,
        {
            "id": problem,
            "status": status,
            "title": "refused",
            "detail": "what the corpus scripted",
            "type": f"https://hook0.com/documentation/errors/{problem}",
        },
        headers=headers if headers is not None else {},
    )


def issued_for(api: FakeHook0Api, caller: Caller, refusal: ScriptedResponse) -> tuple[int, bool]:
    """How many requests a send made when the API answered that way and then took the event."""
    api.will_answer(refusal, ingested(INGESTED_ID))

    try:
        caller(prompt_options(max_attempts=4)).send_event(an_event())
    except Hook0ClientError:
        return len(api.received), False
    return len(api.received), True


def paced_pair() -> tuple[dict[str, Any], dict[str, Any]]:
    """Two problems answering the same status, one worth repeating and one not.

    That pair is the whole reason the corpus classifies problems rather than statuses, and the
    retryable one is the answer the API names a delay beside.
    """
    for rule in RETRY["problems"]:
        if not rule["retryable"]:
            continue
        for other in RETRY["problems"]:
            if other["status"] == rule["status"] and not other["retryable"]:
                return rule, other
    raise AssertionError("no status of the corpus carries opposite verdicts")


@pytest.mark.parametrize("rule", RETRY["problems"], ids=lambda rule: rule["problem"])
def test_the_corpus_says_what_every_problem_does_to_a_send(
    api: FakeHook0Api, caller: Caller, rule: dict[str, Any]
) -> None:
    """One send per problem the API can report.

    The status is not what decides: the corpus carries problems answering the same status with
    opposite verdicts, and a client reading the status alone fails half of them.
    """
    issued, ingested_event = issued_for(api, caller, answered(rule["status"], rule["problem"]))

    expected = 2 if rule["retryable"] else 1
    assert issued == expected, (
        f"`{rule['problem']}` under {rule['status']} issued {issued} requests where the corpus "
        f"expects {expected}: {rule['reason']}"
    )
    assert ingested_event == rule["retryable"]


@pytest.mark.parametrize("rule", RETRY["statuses"], ids=lambda rule: str(rule["status"]))
def test_the_corpus_says_what_every_status_does_to_a_send(
    api: FakeHook0Api, caller: Caller, rule: dict[str, Any]
) -> None:
    """One send per status the API answers, with a body naming no problem this client could read.

    That is also what an older client meets when the API names a problem it has never heard of.
    """
    issued, _ = issued_for(api, caller, answered(rule["status"], "AProblemThisClientHasNeverHeardOf"))

    expected = 2 if rule["retryable"] else 1
    assert issued == expected, (
        f"a status of {rule['status']} issued {issued} requests where the corpus expects {expected}: {rule['reason']}"
    )


def provoked(api: FakeHook0Api, awaiting: bool, cause: str) -> tuple[bool, str]:
    """Makes this client meet one of the causes the corpus names, for real.

    The API takes the event on its second answer, so a cause the corpus calls retryable is one the
    send comes back from. What is answered is whether it survived, and what it said when it did not.
    """
    if cause == "no_answer":
        api.will_answer(ScriptedResponse(201, ingested(INGESTED_ID).body, held_for=1.0), ingested(INGESTED_ID))
        sender = Caller(api.base_url, prompt_options(max_attempts=4, request_timeout=0.2), awaiting)
    elif cause == "answer_above_a_bound":
        api.will_answer(ScriptedResponse(200, "x" * 1024), ingested(INGESTED_ID))
        sender = Caller(api.base_url, prompt_options(max_attempts=4, max_response_bytes=64), awaiting)
    elif cause == "unusable_api_url":
        # Nothing listens, and nothing is sent: the URL names no scheme a request could travel on.
        sender = Caller("gopher://nowhere.invalid", prompt_options(max_attempts=4), awaiting)
    else:
        raise AssertionError(f"the corpus names the transport cause `{cause}`, which this suite cannot provoke")

    try:
        sender.send_event(an_event())
    except Hook0ClientError as refused:
        return False, str(refused)
    return True, ""


@pytest.mark.parametrize("rule", RETRY["transport"]["causes"], ids=lambda rule: rule["cause"])
def test_the_corpus_says_what_every_transport_cause_does_to_a_send(
    api: FakeHook0Api, awaiting: bool, rule: dict[str, Any]
) -> None:
    """Every failure that produced no answer to read.

    They arrive as one exception in this client as in most runtimes, and only one of them could end
    differently: a client deciding by the type spends four attempts on a mistyped API URL and then
    hands its caller a message that accuses the network.
    """
    survived, refusal = provoked(api, awaiting, rule["cause"])

    if rule["retryable"]:
        assert survived, f"a send that met `{rule['cause']}` gave up (`{refusal}`): {rule['reason']}"
        return

    assert not survived, f"a send that met `{rule['cause']}` reported success: {rule['reason']}"
    # A send that gave up after more than one attempt says so, and this one must not have.
    assert "gave up after" not in refusal, (
        f"`{rule['cause']}` was met more than once (`{refusal}`) where the corpus says repeating it "
        f"changes nothing: {rule['reason']}"
    )


def padding(lines: int, each: int) -> dict[str, str]:
    """Filler an answer's head is scripted with: `lines` of them, `each` bytes of name and value."""
    return {f"X-Filler-{index}": "v" * max(each - len(f"X-Filler-{index}"), 1) for index in range(lines)}


# The shapes an abusive head comes in, each one crossing exactly one of the ceilings the corpus
# names, and what its refusal has to say for the ceiling it crossed to be the one it was built for.
#
# The whole-head shape is a few wide lines rather than many narrow ones. It is the only one of the
# three that bounds what a head costs, since a line count and a size per line multiply and neither
# refuses a head that stays under both; eight lines a quarter of that ceiling each weigh twice the
# ceiling while sitting nowhere near the other two. A shape just inside the line count would be
# refused on the count instead — the answer's own `Server`, `Date`, `Content-Type`, `Content-Length`
# and `Connection` are counted beside the filler — and a refusal obtained that way would have proved
# the opposite of what the case is for.
#
# The per-line shape is read for the number it names rather than for a wording, because the refusal
# comes from whatever reads the line: the line reader of one flavour and the standard library of the
# other both draw the line at that same number, and each says so in its own words.
ABUSIVE_HEADS = {
    "more headers than are read": (
        padding(BOUNDS["max_response_headers"] + 8, 16),
        f"more than the {BOUNDS['max_response_headers']} headers read at most",
    ),
    "a header longer than is read": (
        padding(1, BOUNDS["max_header_bytes"] + 8),
        str(BOUNDS["max_header_bytes"]),
    ),
    "a whole head longer than is read": (
        padding(8, BOUNDS["max_head_bytes"] // 4),
        f"a head above the {BOUNDS['max_head_bytes']} bytes read at most",
    ),
}


@pytest.mark.parametrize("head", list(ABUSIVE_HEADS))
def test_a_head_above_the_ceilings_the_corpus_names_is_refused(api: FakeHook0Api, caller: Caller, head: str) -> None:
    """The head of an answer is held to the same bounds as its body.

    It is written by the other end, so a client that bounds the body and not the headers has only
    moved where a server spends its caller's memory.
    """
    headers, says = ABUSIVE_HEADS[head]
    api.will_answer(ScriptedResponse(200, {}, headers=headers), ingested(INGESTED_ID))

    with pytest.raises(Hook0ClientError) as refused:
        caller(prompt_options(max_attempts=4)).send_event(an_event())

    assert says in str(refused.value), (
        f"a head built to cross the ceiling `{says}` was refused as `{refused.value}`, which names another one"
    )
    assert "gave up after" not in str(refused.value), "an answer this client will not read was drawn again"


def test_an_answer_whose_whole_head_is_well_under_the_ceiling_is_read(api: FakeHook0Api, caller: Caller) -> None:
    """A head weighing half of what a head may weigh, which every runtime reads.

    Only the refusal above the bound is a property this client owns. Whether a head just under it is
    read at all is settled by the runtime before this client is consulted, and the strictest runtime
    any target runs on draws its own line a little above the number the corpus names — so the
    accepting side is exercised well clear of that band, and the band itself is left untested rather
    than pinned to the build of the day.
    """
    narrow = padding(8, BOUNDS["max_head_bytes"] // 16)
    api.will_answer(ScriptedResponse(201, ingested(INGESTED_ID).body, headers=narrow))

    assert caller(prompt_options(max_attempts=4)).send_event(an_event()) == INGESTED_ID
    assert len(api.received) == 1


def test_every_request_carries_what_the_corpus_says_it_does(api: FakeHook0Api, caller: Caller) -> None:
    """What actually reached the socket, read back off it.

    A representation a client forgets to ask for costs nothing until the API serves a second one, at
    which point it costs everything, which is exactly the kind of divergence nobody notices by hand.
    """
    api.will_answer(ingested(INGESTED_ID))
    caller(prompt_options(max_attempts=4)).send_event(an_event())

    # A send carries a body, so every occasion the corpus declares applies to this one request.
    carried = api.received[0].headers
    bound = {"token": "token-xyz", "language": "python", "version": VERSION}
    for header in REQUEST["headers"]:
        template = header["value"]
        arrived = carried.get(header["name"].lower(), "")
        chunks = template_chunks(template, bound)

        assert matches_chunks(chunks, arrived), (
            f"the request carried `{header['name']}: {arrived}` where the shared contract says "
            f"`{template}`: {header['reason']}"
        )

        # A value with a hole this suite cannot fill is one the client composed out of what the
        # platform told it, and what the platform says is as long as it feels like.
        if len(chunks) > 1:
            written = len(arrived.encode("utf-8"))
            assert written <= REQUEST["max_composed_bytes"], (
                f"the request carried {written} bytes of `{header['name']}`, above the "
                f"{REQUEST['max_composed_bytes']} the shared contract cuts a composed value to"
            )


@pytest.mark.parametrize("delay", RETRY["retry_after"]["cases"], ids=lambda delay: delay["name"])
def test_the_delay_the_api_names_is_honoured_and_bounded(
    api: FakeHook0Api, caller: Caller, delay: dict[str, Any]
) -> None:
    """Every value of the delay header the corpus carries.

    The header is written by the other end, so honouring it whole would hand a stranger the length
    of this client's send. What the corpus asks for is that a delay be waited out when the budget
    can afford it and cut down to what is left of the budget when it cannot.
    """
    paced, _ = paced_pair()
    api.will_answer(
        answered(paced["status"], paced["problem"], {RETRY["retry_after"]["header"]: delay["header"]}),
        ingested(INGESTED_ID),
    )
    options = Hook0ClientOptions(
        retry_policy=RetryPolicy(4, PROMPT_BACKOFF, PROMPT_BACKOFF, DELAY_BUDGET),
        request_timeout=5.0,
    )

    started = time.monotonic()
    caller(options).send_event(an_event())
    waited = time.monotonic() - started

    assert len(api.received) == 2, "a paced answer was not retried"

    expected = min(float(delay["seconds"]), DELAY_BUDGET) if delay["honoured"] else 0.0
    assert waited >= expected, (
        f"`{RETRY['retry_after']['header']}: {delay['header']}` was retried after {waited:.3f}s, "
        f"sooner than the {expected:.3f}s it asked for"
    )
    assert waited <= expected + DELAY_SLACK, (
        f"`{RETRY['retry_after']['header']}: {delay['header']}` held the send for {waited:.3f}s, "
        f"above the {expected:.3f}s it is bounded to"
    )


def test_the_bounds_are_the_ones_the_corpus_names() -> None:
    """This client's defaults, held against the one place the numbers are written down.

    Both sides are discovered: the names come out of the corpus and the values out of this client,
    so a ceiling added there and applied nowhere here is named as missing rather than quietly
    skipped by a case that only ever checks the ones it already knew about.
    """
    from hook0.client import MAX_ATTEMPTS_CAP
    from hook0.transport import MAX_HEADERS, MAX_HEAD_BYTES, MAX_LINE_BYTES

    client = Hook0Client("http://127.0.0.1:1", "app-123", "token-xyz")
    policy = client.options.retry_policy
    applied = {
        "max_attempts": policy.max_attempts,
        "max_attempts_cap": MAX_ATTEMPTS_CAP,
        "initial_backoff_ms": policy.initial_backoff * 1000,
        "max_backoff_ms": policy.max_backoff * 1000,
        "max_total_delay_ms": policy.max_total_delay * 1000,
        "request_timeout_ms": client.options.request_timeout * 1000,
        "max_payload_bytes": client.options.max_payload_bytes,
        "max_response_bytes": client.options.max_response_bytes,
        "max_response_headers": MAX_HEADERS,
        "max_header_bytes": MAX_LINE_BYTES,
        "max_head_bytes": MAX_HEAD_BYTES,
    }

    unapplied = sorted(set(BOUNDS) - set(applied))
    assert not unapplied, f"the corpus names bounds this client does not apply: {', '.join(unapplied)}"
    for name, wanted in BOUNDS.items():
        assert applied[name] == wanted, name


def test_every_refusal_the_corpus_declares_reads_as_one_of_this_client_s() -> None:
    """A refusal named in the corpus and mapped to nothing here would pass under any wording."""
    assert set(SIGNATURE["refusals"]) <= set(REFUSALS)


@pytest.mark.parametrize("vector", SIGNATURE["vectors"], ids=lambda vector: vector["name"])
def test_every_delivery_of_the_corpus_is_verified_as_it_says(vector: dict[str, Any]) -> None:
    """Every signature vector the corpus carries.

    A refused delivery has to be refused for the reason the corpus names: a client that computed a
    code over a header that never arrived and reported a mismatch would otherwise look right.
    """
    current_time = datetime.datetime.fromtimestamp(vector["current_time"], tz=datetime.UTC)
    delivered = [(name, value) for name, value in vector["headers"]]

    def verify() -> bool:
        return verify_webhook_signature_with_current_time(
            vector["signature"],
            vector["payload"].encode("utf-8"),
            delivered,
            vector["secret"],
            float(vector["tolerance_seconds"]),
            current_time,
        )

    if vector["verdict"] == "accepted":
        assert verify(), vector["reason"]
        return

    with pytest.raises(Hook0ClientError) as refused:
        verify()
    assert REFUSALS[vector["refusal"]] in str(refused.value), (
        f"a delivery the corpus refuses as `{vector['refusal']}` was answered `{refused.value}`: {vector['reason']}"
    )

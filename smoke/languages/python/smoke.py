"""The Python client against a Hook0 that is really running.

Two things happen here, and the second is the reason the first is worth having.

The control: whether an application secret the API minted is accepted, whether a second send under
an identifier already ingested is reported as the conflict it is, and whether a signature the output
worker computed verifies. Those are the three questions no loopback suite can ask itself, because a
suite that signs and verifies with the same module only proves the module agrees with itself.

The surface: every operation the API document declares, driven through the generated layer against
the same instance, and every model type it decodes out of a real answer. `clients/python/tests`
already drives all of them — against an API the suite itself writes, out of the same document the
client was generated from. That proves the client matches the document. It cannot prove the document
matches Hook0, and a field the API really answers under another name passes there and fails on a
consumer's first call.
"""

import datetime
import os
import sys
import time
import uuid
from pathlib import Path
from typing import Any, Sequence, TypeVar
from urllib.parse import urlsplit, urlunsplit

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent.parent.parent / "clients" / "python" / "src"))

from hook0 import Event, Hook0Client, Hook0ClientError, HttpTransport  # noqa: E402
from hook0 import verify_webhook_signature  # noqa: E402
from hook0.generated import (  # noqa: E402
    ApplicationPost,
    ApplicationSecretPost,
    ApplicationSecretsApi,
    ApplicationsApi,
    ErrorsApi,
    EventPost,
    EventsApi,
    EventsPerDayApi,
    EventTypePost,
    EventTypesApi,
    InstanceApi,
    PayloadContentTypesApi,
    QuotasApi,
    ReplayEvent,
    RequestAttemptsApi,
    ResponseApi,
    ServiceTokenApi,
    ServiceTokenPost,
    SubscriptionPost,
    SubscriptionPostTarget,
    SubscriptionsApi,
)
from hook0.generated.errors import ProblemError  # noqa: E402

# The conflict the API answers a duplicated ingestion with.
ALREADY_INGESTED = "EventAlreadyIngested"

# What this smoke labels everything it creates with, so that the subscription it makes and the event
# it sends find each other.
LANGUAGE = "python"

# Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
# delivery proves is proved once, by the webhook the harness catches and every language verifies.
NOWHERE = "http://127.0.0.1:1/"

# What a paced instance answers.
TOO_MANY_REQUESTS = 429

# The most times one request is sent again after the instance answered that it is arriving too fast,
# and the shortest and longest this waits in between.
PACED_AGAIN = 8
SHORTEST_PAUSE = 0.2
LONGEST_PAUSE = 10.0

T = TypeVar("T")


def setting(name: str) -> str:
    """A setting the harness passes, or a refusal naming it."""
    value = os.environ.get(name, "")
    if not value:
        raise SystemExit(f"{name} is not set")
    return value


def origin_of(api_url: str) -> str:
    """The instance without the path the hand-written half is built with.

    The generated half composes paths that already carry `/api/v1`, since the API document's own
    server URL is the bare origin. Handing this client's transport the whole of `HOOK0_API_URL`
    happens to reach the same request: it resolves with `urljoin`, so an absolute path replaces the
    base's as RFC 3986 says, and what the base carried is discarded whichever of the two it was
    given. That is how one language joins two URLs rather than a contract — the TypeScript client
    resolved its base with `new URL` and was posting to `/api/event` until the first live run found
    it — so this points at the origin, which is what the contract says.
    """
    parts = urlsplit(api_url)
    return urlunsplit((parts.scheme, parts.netloc, "", "", ""))


class Paced:
    """The transport every generated method is issued through, waiting out a paced instance.

    Hook0 paces callers per credential, and a flow driving three dozen operations one after another
    is exactly what that is for. The answer says the request was not processed and is safe to send
    again after the delay it names, so this waits and sends it again rather than handing the caller
    a problem that says nothing about the operation it was asking about.

    It wraps the transport the package ships rather than replacing it: `deliver` is what that
    transport offers a caller who needs what the answer carried beside its body, which is precisely
    the delay. Bounded both ways — an instance still refusing after this many tries is one whose
    answer the caller should see.
    """

    def __init__(self, inner: HttpTransport) -> None:
        self._inner = inner

    def request(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]],
        body: Any = None,
    ) -> tuple[int, bytes]:
        sent = 0
        while True:
            status, headers, payload = self._inner.deliver(method, path, query, body)
            sent += 1
            if status != TOO_MANY_REQUESTS or sent > PACED_AGAIN:
                return status, payload
            time.sleep(pause(headers))


def pause(headers: dict[str, str]) -> float:
    """How long the answer says to wait, held between a floor and a ceiling of this smoke's own.

    The floor is there because the header counts in whole seconds and the delay being waited out is
    a fraction of one, so a truthful `Retry-After: 0` would otherwise mean sending the same request
    again immediately, forever. The ceiling is there because a header is written by a server this
    smoke does not control.
    """
    for name, value in headers.items():
        if name.lower() != "retry-after":
            continue
        try:
            seconds = float(value.strip())
        except ValueError:
            break
        return min(max(seconds, SHORTEST_PAUSE), LONGEST_PAUSE)
    return SHORTEST_PAUSE


def read(operation: str, answering: Any) -> Any:
    """One operation the flow goes on to use the answer of, which therefore has to be a success."""
    try:
        answered = answering()
    except ProblemError as refused:
        raise SystemExit(
            f"{operation}: the flow needs what it answers, and it answered {refused}"
        ) from refused
    print(f"exercised {operation} accepted")
    return answered


def exercised(operation: str, answering: Any) -> None:
    """One operation driven for its own sake, reported whichever way the instance answered it.

    A success and a problem are both complete round trips through the generated layer: the request
    was composed, the instance answered, and this client read the answer. What is neither — the API
    not reached, a body this client cannot read, a problem it does not know — stops the smoke,
    because none of those say the client and the instance agree on anything.
    """
    try:
        answering()
    except ProblemError as refused:
        if refused.problem is None:
            raise SystemExit(
                f"{operation}: the instance answered {refused.status} and this client read no "
                f"problem it knows out of it: {refused}"
            ) from refused
        print(f"exercised {operation} refused:{refused.problem.id_.value}")
        return
    print(f"exercised {operation} accepted")


def decoded(model: str, value: T) -> T:
    """Reports one generated model type as decoded out of a real answer.

    The value is taken and handed back rather than only named, so the line cannot outlive what it is
    about: a type that stops being part of an answer stops resolving here.
    """
    print(f"decoded {model}")
    return value


def event(event_type: str, event_id: str | None) -> Event:
    """The event both control sends carry, under the identifier the caller names."""
    return Event(
        event_type=event_type,
        payload='{"from":"the python smoke"}',
        payload_content_type="application/json",
        labels={"language": LANGUAGE},
        event_id=event_id,
    )


def control(api_url: str, application_id: str, token: str, event_type: str) -> None:
    """The three questions the loopback suite cannot ask, against the instance that can answer."""
    client = Hook0Client(api_url, application_id, token)

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


def surface() -> None:
    """Every operation the API document declares, in the order a consumer would drive them.

    Two credentials, because the API takes two and one of them cannot do everything. An application
    secret is scoped to the application it belongs to; what belongs to the organization — listing its
    applications, everything about service tokens, its per-day counts — needs the organization-scoped
    token beside it.
    """
    origin = origin_of(setting("HOOK0_API_URL"))
    application = setting("HOOK0_APPLICATION_ID")
    organization = setting("HOOK0_ORGANIZATION_ID")
    seeded = setting("HOOK0_SEEDED_APPLICATION_ID")
    attempt = setting("HOOK0_REQUEST_ATTEMPT_ID")
    response = setting("HOOK0_RESPONSE_ID")

    held = Paced(HttpTransport(origin, setting("HOOK0_TOKEN")))
    organization_wide = Paced(HttpTransport(origin, setting("HOOK0_SERVICE_TOKEN")))

    applications = ApplicationsApi(held)
    secrets = ApplicationSecretsApi(held)
    event_types = EventTypesApi(held)
    subscriptions = SubscriptionsApi(held)
    events = EventsApi(held)
    events_per_day = EventsPerDayApi(held)
    instance = InstanceApi(held)
    quotas = QuotasApi(held)
    payload_content_types = PayloadContentTypesApi(held)
    errors = ErrorsApi(held)

    organization_applications = ApplicationsApi(organization_wide)
    organization_events_per_day = EventsPerDayApi(organization_wide)
    request_attempts = RequestAttemptsApi(organization_wide)
    responses = ResponseApi(organization_wide)
    service_tokens = ServiceTokenApi(organization_wide)

    # What the instance says about itself, which is what an application asks before it has anything
    # of its own: how it is configured, what it will let this account do, what a payload may be, and
    # every problem it can report.
    decoded("InstanceConfig", read("instance.get", instance.get))

    allowed = read("quotas.get", quotas.get)
    decoded("QuotasResponseLimits", allowed.limits)
    decoded("QuotasResponse", allowed)

    exercised("payload_content_types.list", payload_content_types.list)

    catalogue = read("errors.list", errors.list)
    if not catalogue:
        raise SystemExit("the instance published an empty catalogue of the problems it can report")
    decoded("ProblemId", catalogue[0].id_)
    decoded("Problem", catalogue[0])

    # The application this smoke owns. One per language, so that the three deletions at the end of
    # this flow are real deletions rather than something eleven other smokes have to live with.
    info = read("applications.get", lambda: applications.get(application))
    decoded("ApplicationInfoConsumption", info.consumption)
    decoded("ApplicationInfoQuotas", info.quotas)
    decoded("ApplicationInfoOnboardingStepsEvent", info.onboarding_steps.event)
    decoded("ApplicationInfoOnboardingStepsEventType", info.onboarding_steps.event_type)
    decoded("ApplicationInfoOnboardingStepsSubscription", info.onboarding_steps.subscription)
    decoded("ApplicationInfoOnboardingSteps", info.onboarding_steps)
    decoded("ApplicationInfo", info)

    renamed = read(
        "applications.update",
        lambda: applications.update(
            application,
            ApplicationPost(
                name="the application the python smoke drives",
                organization_id=uuid.UUID(organization),
            ),
        ),
    )
    decoded("Application", renamed)

    # The organization's, so the organization credential. Listing what an account has is the first
    # thing a console does.
    exercised("applications.list", lambda: organization_applications.list(organization))

    # This one is driven with the *application* secret on purpose, and it is the flow's one refusal.
    # Creating an application is the organization's business and an application secret is not the
    # organization's, so the instance answers a problem document and this client reads it — which is
    # the half of the client that nothing else here would exercise.
    exercised(
        "applications.create",
        lambda: applications.create(
            ApplicationPost(
                name="an application the python smoke's application secret may not create",
                organization_id=uuid.UUID(organization),
            )
        ),
    )

    # A second secret, so that the one this smoke is authenticating with is never the one it
    # revokes. Deleting that one succeeds and then locks the flow out of everything below.
    minted = read(
        "applicationSecrets.create",
        lambda: secrets.create(
            ApplicationSecretPost(
                application_id=uuid.UUID(application),
                name="a secret the python smoke minted",
            )
        ),
    )
    decoded("ApplicationSecret", minted)

    exercised("applicationSecrets.read", lambda: secrets.read(application))
    exercised(
        "applicationSecrets.update",
        lambda: secrets.update(
            str(minted.token),
            ApplicationSecretPost(
                application_id=uuid.UUID(application),
                name="a secret the python smoke renamed",
            ),
        ),
    )
    exercised(
        "applicationSecrets.delete",
        lambda: secrets.delete(str(minted.token), application),
    )

    # An event type of this smoke's own, rather than the one the harness declared: what is created
    # here is what is subscribed to, sent, replayed and deleted below.
    declared = read(
        "eventTypes.create",
        lambda: event_types.create(
            EventTypePost(
                application_id=uuid.UUID(application),
                resource_type="smoke",
                service=LANGUAGE,
                verb="ran",
            )
        ),
    )
    decoded("EventType", declared)

    exercised(
        "eventTypes.get",
        lambda: event_types.get(declared.event_type_name, application),
    )
    exercised("eventTypes.list", lambda: event_types.list(application))

    labels = {"language": LANGUAGE}
    target = SubscriptionPostTarget(headers={}, method="POST", type_="http", url=NOWHERE)
    subscription = read(
        "subscriptions.create",
        lambda: subscriptions.create(
            SubscriptionPost(
                application_id=uuid.UUID(application),
                event_types=[declared.event_type_name],
                is_enabled=True,
                target=target,
                description="what the python smoke subscribes to its own events with",
                labels=labels,
            )
        ),
    )
    decoded("SubscriptionTarget", subscription.target)
    decoded("Subscription", subscription)
    subscribed = str(subscription.subscription_id)

    exercised("subscriptions.get", lambda: subscriptions.get(subscribed))
    exercised("subscriptions.list", lambda: subscriptions.list(application))
    exercised(
        "subscriptions.update",
        lambda: subscriptions.update(
            subscribed,
            SubscriptionPost(
                application_id=uuid.UUID(application),
                event_types=[declared.event_type_name],
                is_enabled=True,
                target=target,
                description="what the python smoke renamed it to",
                labels=labels,
            ),
        ),
    )

    # The event the subscription above selects, sent through the generated layer rather than through
    # `send_event`: the hand-written half has its own three questions above, and this is the
    # operation the document declares.
    ingested = read(
        "events.ingest",
        lambda: events.ingest(
            EventPost(
                application_id=uuid.UUID(application),
                event_type=declared.event_type_name,
                labels=labels,
                occurred_at=datetime.datetime.now(datetime.timezone.utc),
                payload='{"from":"the python smoke"}',
                payload_content_type="application/json",
                event_id=uuid.uuid4(),
            )
        ),
    )
    decoded("IngestedEvent", ingested)
    sent = str(ingested.event_id)

    decoded("EventWithPayload", read("events.get", lambda: events.get(sent, application)))

    listed = read("events.list", lambda: events.list(application))
    if not listed:
        raise SystemExit("the instance ingested an event and then listed none")
    decoded("Event", listed[0])

    exercised(
        "events.replay",
        lambda: events.replay(sent, ReplayEvent(application_id=uuid.UUID(application))),
    )

    # This application was created a moment ago and the counts come out of a view the instance
    # refreshes on a cycle of its own, so this answers a list with nothing in it — which is an
    # answer, and one a client has to be able to read.
    exercised(
        "events_per_day.list_for_application",
        lambda: events_per_day.list_for_application(application),
    )

    # The organization's counts do have something in them: the harness waited for the instance to
    # refresh them before running any of this, precisely so that the type they are answered with is
    # one a client decodes rather than one nothing ever produces.
    per_day = read(
        "events_per_day.list_for_organization",
        lambda: organization_events_per_day.list_for_organization(organization),
    )
    if not per_day:
        raise SystemExit("the organization has ingested events and its per-day counts are empty")
    decoded("EventsPerDayEntry", per_day[0])

    # An attempt and a response exist only once the output worker has finished a delivery. The
    # harness waited for one, in the application it caught the shared delivery from, and handed the
    # ids on — so this reads them back with the organization credential rather than waiting again.
    exercised("requestAttempts.read", lambda: request_attempts.read(seeded))
    attempted = read("requestAttempts.get", lambda: request_attempts.get(attempt, seeded))
    decoded("RequestAttemptEvent", attempted.event)
    decoded("RequestAttemptSubscription", attempted.subscription)
    decoded("RequestAttemptStatusType", attempted.status.type_)
    decoded("RequestAttemptStatus", attempted.status)
    decoded("RequestAttempt", attempted)

    decoded("Response", read("response.get", lambda: responses.get(response, seeded)))

    # Service tokens belong to the organization, so they are minted, read and revoked with the
    # organization credential. The one revoked below is the one minted here — never the one this
    # half of the flow is authenticating with.
    token = read(
        "serviceToken.create",
        lambda: service_tokens.create(
            ServiceTokenPost(
                name="a token the python smoke minted",
                organization_id=uuid.UUID(organization),
            )
        ),
    )
    decoded("ServiceToken", token)
    minted_id = str(token.token_id)

    exercised("serviceToken.list", lambda: service_tokens.list(organization))
    exercised("serviceToken.get", lambda: service_tokens.get(minted_id, organization))
    exercised(
        "serviceToken.edit",
        lambda: service_tokens.edit(
            minted_id,
            ServiceTokenPost(
                name="a token the python smoke renamed",
                organization_id=uuid.UUID(organization),
            ),
        ),
    )
    exercised(
        "serviceToken.delete",
        lambda: service_tokens.delete(minted_id, organization),
    )

    # Destroyed in the order the instance can accept: the subscription that references the event
    # type, then the event type, then the application — which is last because the secret this whole
    # flow authenticates with stops authenticating the moment its application is gone.
    exercised(
        "subscriptions.delete",
        lambda: subscriptions.delete(subscribed, application),
    )
    exercised(
        "eventTypes.delete",
        lambda: event_types.delete(declared.event_type_name, application),
    )
    exercised("applications.delete", lambda: applications.delete(application))


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
    control(
        setting("HOOK0_API_URL"),
        setting("HOOK0_APPLICATION_ID"),
        setting("HOOK0_TOKEN"),
        setting("HOOK0_EVENT_TYPE"),
    )
    surface()

    # Last, and on purpose: it needs no instance at all, so it still answers after the flow above
    # has deleted the application it was run against.
    verify(Path(setting("HOOK0_DELIVERY")))
    print("the signature the instance produced verifies")


main()

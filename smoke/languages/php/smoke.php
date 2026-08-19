<?php

declare(strict_types=1);

// The PHP client against a Hook0 that is really running.
//
// Two things happen here, and the second is the reason the first is worth having.
//
// The control: whether an application secret the API minted is accepted, whether a second send
// under an identifier already ingested is reported as the conflict it is, and whether a signature
// the output worker computed verifies. Those are the three questions no loopback suite can ask
// itself, because a suite that signs and verifies with the same sources only proves the sources
// agree with themselves.
//
// The surface: every operation the API document declares, driven through the generated layer
// against the same instance, and every model type it decodes out of a real answer.
// `clients/php/tests` already drives all of them — against an API the suite itself writes, out of
// the same document the client was generated from. That proves the client matches the document. It
// cannot prove the document matches Hook0, and a field the API really answers under another name
// passes there and fails on a consumer's first call.


use Hook0\Client;
use Hook0\ClientError;
use Hook0\Event;
use Hook0\Generated\ApplicationPost;
use Hook0\Generated\ApplicationSecretPost;
use Hook0\Generated\ApplicationSecretsApi;
use Hook0\Generated\ApplicationsApi;
use Hook0\Generated\ErrorsApi;
use Hook0\Generated\EventPost;
use Hook0\Generated\EventTypePost;
use Hook0\Generated\EventTypesApi;
use Hook0\Generated\EventsApi;
use Hook0\Generated\EventsPerDayApi;
use Hook0\Generated\InstanceApi;
use Hook0\Generated\PayloadContentTypesApi;
use Hook0\Generated\ProblemError;
use Hook0\Generated\QuotasApi;
use Hook0\Generated\RateLimitedError;
use Hook0\Generated\ReplayEvent;
use Hook0\Generated\RequestAttemptsApi;
use Hook0\Generated\ResponseApi;
use Hook0\Generated\ServiceTokenApi;
use Hook0\Generated\ServiceTokenPost;
use Hook0\Generated\SubscriptionPost;
use Hook0\Generated\SubscriptionPostTarget;
use Hook0\Generated\SubscriptionsApi;
use Hook0\Signature;
use Hook0\Transport;

/**
 * Loads the package the way installing it would, out of the source in this repository.
 *
 * The rule is the one `clients/php/composer.json` declares, written out here rather than reached
 * through the autoloader Composer writes. Two reasons, and the second is the one that bites: an
 * installed copy is not what this is meant to exercise, and there is no installed copy to reach —
 * `clients/php/vendor/` is ignored by git and the job that runs this harness installs `php-cli` and
 * `php-curl` and no Composer at all. The client's own suite answers the same question the same way,
 * in `clients/php/tests/bootstrap.php`.
 */
spl_autoload_register(static function (string $declared): void {
    $prefix = 'Hook0\\';
    if (!str_starts_with($declared, $prefix)) {
        return;
    }

    $path = __DIR__ . '/../../../clients/php/src/'
        . str_replace('\\', '/', substr($declared, strlen($prefix))) . '.php';
    if (is_file($path)) {
        require $path;
    }
});

/** The conflict the API answers a duplicated ingestion with. */
const ALREADY_INGESTED = 'EventAlreadyIngested';

/**
 * What this smoke labels everything it creates with, so that the subscription it makes and the
 * event it sends find each other.
 */
const LANGUAGE = 'php';

/**
 * Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
 * delivery proves is proved once, by the webhook the harness catches and every language verifies.
 */
const NOWHERE = 'http://127.0.0.1:1/';

/** The most times one paced request is sent again. */
const PACED_AGAIN = 8;

/** The shortest this waits between two tries, and the longest, in microseconds. */
const SHORTEST_PAUSE = 200_000;
const LONGEST_PAUSE = 2_000_000;

/** A setting the harness passes, or a refusal naming it. */
function setting(string $name): string
{
    $value = getenv($name);
    if ($value === false || $value === '') {
        throw new RuntimeException("$name is not set");
    }
    return $value;
}

/** The event both sends carry, under the identifier the caller names. */
function event(string $eventType, ?string $eventId): Event
{
    return new Event(
        eventType: $eventType,
        payload: '{"from":"the php smoke"}',
        payloadContentType: 'application/json',
        labels: ['language' => LANGUAGE],
        eventId: $eventId,
    );
}

/** The same event, twice, under the identifier the API minted for the first of them. */
function sendTwice(): void
{
    $client = new Client(
        setting('HOOK0_API_URL'),
        setting('HOOK0_APPLICATION_ID'),
        setting('HOOK0_TOKEN'),
    );
    $eventType = setting('HOOK0_EVENT_TYPE');

    $sent = $client->sendEvent(event($eventType, null));
    echo "ingested $sent\n";

    try {
        $client->sendEvent(event($eventType, $sent));
        throw new RuntimeException('sending the same event twice was accepted twice');
    } catch (ClientError $refused) {
        $said = $refused->getMessage();
        if (!str_contains($said, ALREADY_INGESTED)) {
            throw new RuntimeException(
                'the second send failed without naming ' . ALREADY_INGESTED . ": $said"
            );
        }
    }
    echo 'the second send reported ' . ALREADY_INGESTED . "\n";
}

/**
 * Issues one call, waiting out an instance that is pacing this credential.
 *
 * Hook0 paces callers per credential, and a flow driving three dozen operations one after another
 * is exactly what that is for. The answer says the request was not processed and is safe to send
 * again after a delay, so this sends it again rather than reporting a problem that says nothing
 * about the operation it was asking about.
 *
 * Every other language wraps the transport to do this, which is where the delay the answer names
 * can be read. This one cannot: `Transport` is `final`, and the generated groups take that class
 * rather than an interface — so the pacing sits at the call site, and the delay is this smoke's own
 * rather than the instance's. It grows with each try and is held to a ceiling, because what is
 * being waited out is a bucket refilling at a rate this smoke does not know.
 *
 * @param \Closure(): mixed $asking
 */
function paced(\Closure $asking): mixed
{
    for ($sent = 1; ; $sent++) {
        try {
            return $asking();
        } catch (RateLimitedError $paced) {
            if ($sent > PACED_AGAIN) {
                throw $paced;
            }
            usleep(min(SHORTEST_PAUSE * $sent, LONGEST_PAUSE));
        }
    }
}

/**
 * Reports one operation the flow goes on to use the answer of, which has to be a success.
 *
 * @param \Closure(): mixed $asking
 */
function read(string $operation, \Closure $asking): mixed
{
    try {
        $answered = paced($asking);
    } catch (\Throwable $failed) {
        throw new RuntimeException(
            "$operation: the flow needs what it answers, and it answered " . $failed->getMessage(),
            0,
            $failed,
        );
    }

    echo "exercised $operation accepted\n";
    return $answered;
}

/**
 * Reports one operation driven for its own sake, whichever way the instance answered it.
 *
 * A success and a problem are both complete round trips through the generated layer: the request
 * was composed, the instance answered, and this client read the answer. What is neither — the API
 * not reached, a body this client cannot read, a problem it does not know — stops the smoke,
 * because none of those say the client and the instance agree on anything.
 *
 * @param \Closure(): mixed $asking
 */
function exercised(string $operation, \Closure $asking): void
{
    try {
        paced($asking);
    } catch (ProblemError $refused) {
        $problem = $refused->problem;
        if ($problem === null) {
            throw new RuntimeException(
                "$operation: what came back names no problem this client knows: "
                . $refused->getMessage(),
                0,
                $refused,
            );
        }

        echo "exercised $operation refused:{$problem->id->value}\n";
        return;
    } catch (\Throwable $failed) {
        throw new RuntimeException("$operation: " . $failed->getMessage(), 0, $failed);
    }

    echo "exercised $operation accepted\n";
}

/**
 * Reports one generated model type as decoded out of a real answer.
 *
 * The value is taken rather than only named, so the line cannot outlive what it is about. Taking it
 * is not enough on its own here: reading a member this client no longer declares is a warning in
 * PHP and answers `null`, so the line would go on being printed about a type nothing decoded. Every
 * value below is a member the document marks as always there, so `null` is the one thing it cannot
 * be — which is what makes this refuse rather than decorate.
 */
function decoded(string $model, mixed $value): void
{
    if ($value === null) {
        throw new RuntimeException("$model was not decoded out of what the API answered");
    }

    echo "decoded $model\n";
}

/**
 * The instance without the path the hand-written half is built with.
 *
 * The generated half composes paths that already carry `/api/v1`, since the API document's own
 * server URL is the bare origin. Handing this transport the whole of `HOOK0_API_URL` happens to
 * reach the same request, because a path of its own replaces the base's — but that is how one
 * language joins two URLs rather than a contract, and the TypeScript client was posting to
 * `/api/event` until the first live run found it. So this points at the origin, which is what the
 * contract says.
 */
function originOf(string $apiUrl): string
{
    $parts = parse_url($apiUrl);
    if ($parts === false || !isset($parts['scheme'], $parts['host'])) {
        throw new RuntimeException("`$apiUrl` is not somewhere a request can be sent");
    }
    $port = isset($parts['port']) ? ':' . $parts['port'] : '';

    return $parts['scheme'] . '://' . $parts['host'] . $port;
}

/**
 * Every operation the API document declares, driven against the instance in the order a consumer
 * would: what it needs is created, read and listed, updated, and destroyed last.
 *
 * Two credentials, because the API takes two and one of them cannot do everything. An application
 * secret is scoped to the application it belongs to; what belongs to the organization — listing its
 * applications, everything about service tokens, its per-day counts — needs the organization-scoped
 * token beside it.
 */
function surface(): void
{
    $origin = originOf(setting('HOOK0_API_URL'));
    $application = setting('HOOK0_APPLICATION_ID');
    $organization = setting('HOOK0_ORGANIZATION_ID');
    $seeded = setting('HOOK0_SEEDED_APPLICATION_ID');
    $labels = ['language' => LANGUAGE];

    $held = new Transport($origin, setting('HOOK0_TOKEN'));
    $organizationWide = new Transport($origin, setting('HOOK0_SERVICE_TOKEN'));

    $applications = new ApplicationsApi($held);
    $secrets = new ApplicationSecretsApi($held);
    $eventTypes = new EventTypesApi($held);
    $subscriptions = new SubscriptionsApi($held);
    $events = new EventsApi($held);
    $eventsPerDay = new EventsPerDayApi($held);
    $instance = new InstanceApi($held);
    $quotas = new QuotasApi($held);
    $payloadContentTypes = new PayloadContentTypesApi($held);
    $errorCatalogue = new ErrorsApi($held);

    $organizationApplications = new ApplicationsApi($organizationWide);
    $organizationEventsPerDay = new EventsPerDayApi($organizationWide);
    $requestAttempts = new RequestAttemptsApi($organizationWide);
    $responses = new ResponseApi($organizationWide);
    $serviceTokens = new ServiceTokenApi($organizationWide);

    // What the instance says about itself, which is what an application asks before it has anything
    // of its own: how it is configured, what it will let this account do, what a payload may be, and
    // every problem it can report.
    decoded('InstanceConfig', read('instance.get', $instance->get(...)));

    $allowed = read('quotas.get', $quotas->get(...));
    decoded('QuotasResponseLimits', $allowed->limits);
    decoded('QuotasResponse', $allowed);

    exercised('payload_content_types.list', $payloadContentTypes->list(...));

    $catalogue = read('errors.list', $errorCatalogue->list(...));
    if (count($catalogue) === 0) {
        throw new RuntimeException(
            'the instance published an empty catalogue of the problems it can report'
        );
    }
    decoded('ProblemId', $catalogue[0]->id);
    decoded('Problem', $catalogue[0]);

    // The application this smoke owns. One per language, so that the three deletions at the end of
    // this flow are real deletions rather than something eleven other smokes have to live with.
    $info = read('applications.get', static fn() => $applications->get($application));
    decoded('ApplicationInfoConsumption', $info->consumption);
    decoded('ApplicationInfoQuotas', $info->quotas);
    decoded('ApplicationInfoOnboardingStepsEvent', $info->onboardingSteps->event);
    decoded('ApplicationInfoOnboardingStepsEventType', $info->onboardingSteps->eventType);
    decoded('ApplicationInfoOnboardingStepsSubscription', $info->onboardingSteps->subscription);
    decoded('ApplicationInfoOnboardingSteps', $info->onboardingSteps);
    decoded('ApplicationInfo', $info);

    decoded('Application', read('applications.update', static fn() => $applications->update(
        $application,
        new ApplicationPost(
            name: 'the application the php smoke drives',
            organizationId: $organization,
        ),
    )));

    // The organization's, so the organization credential. Listing what an account has is the first
    // thing a console does.
    exercised(
        'applications.list',
        static fn() => $organizationApplications->list($organization),
    );

    // This one is driven with the *application* secret on purpose, and it is the flow's one refusal.
    // Creating an application is the organization's business and an application secret is not the
    // organization's, so the instance answers a problem document and this client reads it — which is
    // the half of the client that nothing else here would exercise.
    exercised('applications.create', static fn() => $applications->create(new ApplicationPost(
        name: "an application the php smoke's application secret may not create",
        organizationId: $organization,
    )));

    // A second secret, so that the one this smoke is authenticating with is never the one it
    // revokes. Deleting that one succeeds and then locks the flow out of everything below.
    $minted = read('applicationSecrets.create', static fn() => $secrets->create(
        new ApplicationSecretPost(
            applicationId: $application,
            name: 'a secret the php smoke minted',
        ),
    ));
    decoded('ApplicationSecret', $minted);

    exercised('applicationSecrets.read', static fn() => $secrets->read($application));
    exercised('applicationSecrets.update', static fn() => $secrets->update(
        $minted->token,
        new ApplicationSecretPost(
            applicationId: $application,
            name: 'a secret the php smoke renamed',
        ),
    ));
    exercised(
        'applicationSecrets.delete',
        static fn() => $secrets->delete($minted->token, $application),
    );

    // An event type of this smoke's own, rather than the one the harness declared: what is created
    // here is what is subscribed to, sent, replayed and deleted below.
    $declared = read('eventTypes.create', static fn() => $eventTypes->create(new EventTypePost(
        applicationId: $application,
        resourceType: 'smoke',
        service: LANGUAGE,
        verb: 'ran',
    )));
    decoded('EventType', $declared);

    exercised(
        'eventTypes.get',
        static fn() => $eventTypes->get($declared->eventTypeName, $application),
    );
    exercised('eventTypes.list', static fn() => $eventTypes->list($application));

    // An object rather than an empty list: the API reads the headers of a target as a mapping, and
    // an empty PHP array is written as `[]`.
    $target = new SubscriptionPostTarget(
        headers: new \stdClass(),
        method: 'POST',
        type: 'http',
        url: NOWHERE,
    );
    $subscription = read('subscriptions.create', static fn() => $subscriptions->create(
        new SubscriptionPost(
            applicationId: $application,
            eventTypes: [$declared->eventTypeName],
            isEnabled: true,
            target: $target,
            description: 'what the php smoke subscribes to its own events with',
            labels: $labels,
        ),
    ));
    decoded('SubscriptionTarget', $subscription->target);
    decoded('Subscription', $subscription);

    exercised(
        'subscriptions.get',
        static fn() => $subscriptions->get($subscription->subscriptionId),
    );
    exercised('subscriptions.list', static fn() => $subscriptions->list($application));
    exercised('subscriptions.update', static fn() => $subscriptions->update(
        $subscription->subscriptionId,
        new SubscriptionPost(
            applicationId: $application,
            eventTypes: [$declared->eventTypeName],
            isEnabled: true,
            target: $target,
            description: 'what the php smoke renamed it to',
            labels: $labels,
        ),
    ));

    // The event the subscription above selects, sent through the generated layer rather than through
    // sendEvent: the hand-written half has its own three questions above, and this is the operation
    // the document declares.
    $ingested = read('events.ingest', static fn() => $events->ingest(new EventPost(
        applicationId: $application,
        eventType: $declared->eventTypeName,
        labels: $labels,
        occurredAt: new \DateTimeImmutable('now', new \DateTimeZone('UTC')),
        payload: '{"from":"the php smoke"}',
        payloadContentType: 'application/json',
        eventId: Client::generateEventId(),
    )));
    decoded('IngestedEvent', $ingested);

    decoded('EventWithPayload', read(
        'events.get',
        static fn() => $events->get($ingested->eventId, $application),
    ));

    $listed = read('events.list', static fn() => $events->list($application));
    if (count($listed) === 0) {
        throw new RuntimeException('the instance ingested an event and then listed none');
    }
    decoded('Event', $listed[0]);

    exercised('events.replay', static fn() => $events->replay(
        $ingested->eventId,
        new ReplayEvent(applicationId: $application),
    ));

    // This application was created a moment ago and the counts come out of a view the instance
    // refreshes on a cycle of its own, so this answers a list with nothing in it — which is an
    // answer, and one a client has to be able to read.
    exercised(
        'events_per_day.list_for_application',
        static fn() => $eventsPerDay->listForApplication($application),
    );

    // The organization's counts do have something in them: the harness waited for the instance to
    // refresh them before running any of this, precisely so that the type they are answered with is
    // one a client decodes rather than one nothing ever produces.
    $perDay = read(
        'events_per_day.list_for_organization',
        static fn() => $organizationEventsPerDay->listForOrganization($organization),
    );
    if (count($perDay) === 0) {
        throw new RuntimeException(
            'the organization has ingested events and its per-day counts are empty'
        );
    }
    decoded('EventsPerDayEntry', $perDay[0]);

    // An attempt and a response exist only once the output worker has finished a delivery. The
    // harness waited for one, in the application it caught the shared delivery from, and handed the
    // ids on — so this reads them back with the organization credential rather than waiting again.
    exercised('requestAttempts.read', static fn() => $requestAttempts->read($seeded));

    $attempted = read('requestAttempts.get', static fn() => $requestAttempts->get(
        setting('HOOK0_REQUEST_ATTEMPT_ID'),
        $seeded,
    ));
    decoded('RequestAttemptEvent', $attempted->event);
    decoded('RequestAttemptSubscription', $attempted->subscription);
    decoded('RequestAttemptStatusType', $attempted->status->type);
    decoded('RequestAttemptStatus', $attempted->status);
    decoded('RequestAttempt', $attempted);

    decoded('Response', read(
        'response.get',
        static fn() => $responses->get(setting('HOOK0_RESPONSE_ID'), $seeded),
    ));

    // Service tokens belong to the organization, so they are minted, read and revoked with the
    // organization credential. The one revoked below is the one minted here — never the one this
    // half of the flow is authenticating with.
    $issued = read('serviceToken.create', static fn() => $serviceTokens->create(
        new ServiceTokenPost(
            name: 'a token the php smoke minted',
            organizationId: $organization,
        ),
    ));
    decoded('ServiceToken', $issued);

    exercised('serviceToken.list', static fn() => $serviceTokens->list($organization));
    exercised(
        'serviceToken.get',
        static fn() => $serviceTokens->get($issued->tokenId, $organization),
    );
    exercised('serviceToken.edit', static fn() => $serviceTokens->edit(
        $issued->tokenId,
        new ServiceTokenPost(
            name: 'a token the php smoke renamed',
            organizationId: $organization,
        ),
    ));
    exercised(
        'serviceToken.delete',
        static fn() => $serviceTokens->delete($issued->tokenId, $organization),
    );

    // Destroyed in the order the instance can accept: the subscription that references the event
    // type, then the event type, then the application — which is last because the secret this whole
    // flow authenticates with stops authenticating the moment its application is gone.
    exercised('subscriptions.delete', static fn() => $subscriptions->delete(
        $subscription->subscriptionId,
        $application,
    ));
    exercised('eventTypes.delete', static fn() => $eventTypes->delete(
        $declared->eventTypeName,
        $application,
    ));
    exercised('applications.delete', static fn() => $applications->delete($application));
}

/** Verifies what the output worker really delivered, with this client's own verification. */
function verify(string $delivery): void
{
    $read = static fn(string $part): string => (string) file_get_contents($delivery . '/' . $part);

    $headers = [];
    foreach (explode("\n", $read('headers')) as $line) {
        $at = strpos($line, ': ');
        if ($at !== false) {
            $headers[substr($line, 0, $at)] = substr($line, $at + 2);
        }
    }

    Signature::verify(
        trim($read('signature')),
        $read('body'),
        $headers,
        trim($read('secret')),
        (float) trim($read('tolerance')),
    );
}

try {
    sendTwice();
    surface();

    // Last, and on purpose: it needs no instance at all, so it still answers after the flow above
    // has deleted the application it was run against.
    verify(setting('HOOK0_DELIVERY'));
    echo "the signature the instance produced verifies\n";
} catch (\Throwable $refused) {
    fwrite(STDERR, $refused . "\n");
    exit(1);
}

<?php

declare(strict_types=1);

namespace Hook0;

/**
 * The Hook0 client, built once and shared wherever an application sends events.
 *
 * Every event is sent under an identifier this client knows: the one set on the {@see Event}, or a
 * UUIDv7 it generates when the event carries none. Passing none does not mean the identifier comes
 * from the API — the value comes from here, travels with the request, and is what
 * {@see self::sendEvent()} answers.
 *
 * That is what makes retrying safe. The API keys events on that identifier, so a request repeated
 * after a network failure or a server error ingests the event once rather than twice; without a
 * client-chosen identifier, a repeated request would create a second event and deliver it to every
 * subscriber. It also gives the answer to a retry its meaning: `EventAlreadyIngested` in reply to a
 * *repeated* request says an earlier attempt of that same send reached the API, so the send
 * succeeded. The same answer to a *first* attempt is a genuine conflict and is reported as one.
 *
 * Only what could end differently is retried: a request that got no answer, a server error, and an
 * instance saying it is being reached faster than it accepts. What the API refuses outright — a
 * quota that is spent, a payload it will not read — is reported as is, since repeating it would only
 * spend the same round trip again. The verdict for every problem the API can report is written down
 * in the conformance corpus committed beside this package, which the suite here reads.
 *
 * A send is bounded on five axes, each of them the caller's to set: the size of the payload, which
 * is refused before a socket is opened; how long one attempt is given; how many attempts are made;
 * how long a single wait between them may be; and how long every wait of one send may add up to.
 */
final class Client
{
    /** The identifier the API gives the problem it answers when an event identifier is taken. */
    public const ALREADY_INGESTED = 'EventAlreadyIngested';

    /**
     * The identifier the API gives the problem it answers when requests are reaching the instance
     * faster than it accepts them.
     *
     * It shares its status with the quota problems, and is the only one of them worth repeating: a
     * quota clears when a plan changes or a day turns, neither of which happens inside the seconds a
     * send is given, while pacing clears on its own and the answer says when.
     */
    public const RATE_LIMITED = 'RateLimited';

    /** What the API answers when the event identifier a request carries is already taken. */
    private const CONFLICT = 409;

    /**
     * What the API answers both when a quota is spent and when requests are coming in faster than
     * the instance accepts them. Which of the two it is only the problem the body names can say,
     * which is why this status alone decides nothing.
     */
    private const PACED = 429;

    /** First status saying the failure is on the API's side, and so could clear on its own. */
    private const LOWEST_SERVER_ERROR = 500;

    /** Lowest status an answer is read as a success under, and the first one that is no longer one. */
    private const LOWEST_SUCCESS = 200;
    private const LOWEST_REDIRECTION = 300;

    /** What the API names the delay before the request becomes servable in, in whole seconds. */
    private const DELAY_HEADER = 'retry-after';

    /**
     * Longest value of that header read, and the largest delay it may name. A header written by the
     * other end is bounded before it is turned into a number, and a delay above this is one nobody
     * meant.
     */
    private const MAX_DELAY_HEADER_BYTES = 32;
    private const MAX_NAMED_DELAY_SECONDS = (2 ** 31) - 1;

    /**
     * What a whole number of seconds reads as, which is the one form of that header this client
     * honours.
     */
    private const WHOLE_SECONDS = '/^\d+$/';

    /** Where an event is ingested, and where event types are read and created, under the API URL. */
    private const EVENT_PATH = 'event';
    private const EVENT_TYPES_PATH = 'event_types';

    /** What one identifier is made of, and where its pieces sit once it is written out. */
    private const IDENTIFIER_BYTES = 16;
    private const IDENTIFIER_GROUPS = [[0, 8], [8, 4], [12, 4], [16, 4], [20, 12]];

    /**
     * What this client issues its requests through, which is also what a generated operation group
     * is built on.
     */
    public readonly Transport $transport;

    /**
     * @param string $apiUrl base API URL of a Hook0 instance, such as https://app.hook0.com/api/v1
     * @param string $applicationId identifier of the application events are sent to
     * @param string $token an authentication token valid for that application
     * @param Options $options the bounds one send is held to
     */
    public function __construct(
        public readonly string $apiUrl,
        public readonly string $applicationId,
        string $token,
        public readonly Options $options = new Options(),
    ) {
        $this->transport = new Transport(
            $apiUrl,
            $token,
            $options->requestTimeout,
            $options->maxResponseBytes,
            $options->maxResponseHeaders,
            $options->maxHeaderBytes,
            $options->maxHeadBytes
        );
    }

    /**
     * A UUIDv7, the shape of identifier this client mints when it is the one choosing.
     *
     * Its leading 48 bits are the current time in milliseconds, so identifiers generated in sequence
     * are ordered by the moment they were minted, which is what keeps the index they end up in from
     * being written all over. Two minted inside one millisecond share that moment and differ only in
     * what was drawn, so what is ordered is the moment they carry rather than the whole of them.
     */
    public static function generateEventId(): string
    {
        $drawn = random_bytes(self::IDENTIFIER_BYTES);
        $milliseconds = (int) floor(microtime(true) * 1000);

        $bytes = substr(pack('J', $milliseconds), 2, 6) . substr($drawn, 6, 10);
        $bytes[6] = chr((ord($bytes[6]) & 0x0F) | 0x70);
        $bytes[8] = chr((ord($bytes[8]) & 0x3F) | 0x80);

        $written = bin2hex($bytes);
        $groups = [];
        foreach (self::IDENTIFIER_GROUPS as [$offset, $length]) {
            $groups[] = substr($written, $offset, $length);
        }

        return implode('-', $groups);
    }

    /**
     * Sends an event, and answers the identifier it was sent under.
     *
     * @throws ClientError when the event was not ingested
     */
    public function sendEvent(Event $event): string
    {
        $eventId = $this->identifierOf($event);
        $this->refuseOversized($event, $eventId);

        $body = $this->fullEvent($event, $eventId);
        $policy = $this->options->retryPolicy;
        $delays = $policy->delays($this->jitterDraws($policy->attempts() - 1));

        $issued = 0;
        $waited = 0.0;
        while (true) {
            $issued++;
            $outcome = $this->attempt($body);

            if ($outcome->ingested !== null) {
                return $outcome->ingested;
            }
            if ($outcome->alreadyIngested) {
                if ($issued > 1) {
                    return $eventId;
                }

                throw ClientError::eventSending($eventId, $outcome->detail);
            }

            $scheduled = $outcome->retryable ? ($delays[$issued - 1] ?? null) : null;
            if ($scheduled === null) {
                throw $this->givenUp($eventId, $issued, $waited, $outcome->detail);
            }

            $waiting = $this->waitFor($outcome, $scheduled, $policy->maxTotalDelay - $waited);
            usleep((int) round($waiting * 1_000_000));
            $waited += $waiting;
        }
    }

    /**
     * Creates the event types the application does not declare yet, and answers those.
     *
     * @param list<string> $eventTypes
     * @return list<string>
     * @throws ClientError
     */
    public function upsertEventTypes(array $eventTypes): array
    {
        $wanted = array_map(static fn (string $written) => EventType::parse($written), $eventTypes);
        if ($wanted === []) {
            return [];
        }

        $declared = $this->declaredEventTypes();
        $created = [];
        foreach ($wanted as $eventType) {
            if (in_array((string) $eventType, $declared, true)) {
                continue;
            }
            $this->createEventType($eventType);
            $created[] = (string) $eventType;
        }

        return $created;
    }

    /** The identifier an event is sent under: the one it carries, or one generated for it. */
    private function identifierOf(Event $event): string
    {
        $carried = $event->eventId;

        return $carried === null || $carried === '' ? self::generateEventId() : $carried;
    }

    /** Rules an oversized payload out, before a socket is opened for it. */
    private function refuseOversized(Event $event, string $eventId): void
    {
        $size = strlen($event->payload);
        if ($size <= $this->options->maxPayloadBytes) {
            return;
        }

        throw ClientError::payloadTooLarge($eventId, $size, $this->options->maxPayloadBytes);
    }

    /**
     * An event as the API reads one.
     *
     * @return array<string, mixed>
     */
    private function fullEvent(Event $event, string $eventId): array
    {
        $occurredAt = $event->occurredAt ?? new \DateTimeImmutable('now', new \DateTimeZone('UTC'));

        $body = [
            'event_id' => $eventId,
            'application_id' => $this->applicationId,
            'event_type' => $event->eventType,
            'payload' => $event->payload,
            'payload_content_type' => $event->payloadContentType,
            'occurred_at' => Runtime::moment($occurredAt),
            'labels' => Runtime::mapping($event->labels),
        ];
        if ($event->metadata !== null) {
            $body['metadata'] = Runtime::mapping($event->metadata);
        }

        return $body;
    }

    /**
     * One attempt at sending an already-bounded event.
     *
     * @param array<string, mixed> $body
     */
    private function attempt(array $body): Attempt
    {
        try {
            [$status, $headers, $payload] = $this->transport->deliver(
                'POST',
                self::EVENT_PATH,
                [],
                $body
            );
        } catch (TransportError $failure) {
            return new Attempt(null, false, $failure->getMessage(), $failure->retryable, null);
        }

        return $this->readAttempt($status, $headers, $payload);
    }

    /**
     * What the API answered one attempt, and whether repeating it could end differently.
     *
     * @param array<string, string> $headers
     */
    private function readAttempt(int $status, array $headers, string $payload): Attempt
    {
        if ($status >= self::LOWEST_SUCCESS && $status < self::LOWEST_REDIRECTION) {
            $ingested = $this->ingestedId($payload);
            if ($ingested === null) {
                // The API accepted the event but answered something this client cannot read;
                // repeating the request would meet the same answer.
                return new Attempt(
                    null,
                    false,
                    sprintf('the API answered %d without an event id', $status),
                    false,
                    null
                );
            }

            return new Attempt($ingested, false, '', false, null);
        }

        $problem = $this->problemId($payload);
        if ($status === self::CONFLICT && $problem === self::ALREADY_INGESTED) {
            return new Attempt(null, true, $payload, false, null);
        }

        return new Attempt(
            null,
            false,
            $payload,
            $this->isRetryable($status, $problem),
            $this->namedDelay($headers)
        );
    }

    /**
     * Whether repeating a request the API answered that way could end differently.
     *
     * The status decides on its own everywhere but under the one it answers both a spent quota and a
     * paced instance with: a quota clears when a plan changes or a day turns, and neither is
     * something a send spending seconds can wait for. Only the problem the body names tells the two
     * apart, and a body naming a problem this client has never heard of falls back to what the
     * status says.
     */
    private function isRetryable(int $status, ?string $problem): bool
    {
        if ($status === self::PACED) {
            return $problem === self::RATE_LIMITED;
        }

        return $status >= self::LOWEST_SERVER_ERROR;
    }

    /**
     * The delay the API named before the request becomes servable, in seconds.
     *
     * Only a whole number of seconds is read. The header may also carry a date, which is a clock this
     * client would be comparing against its own, and anything else is a header nobody meant: both
     * leave the client's own schedule in place rather than being guessed at.
     *
     * @param array<string, string> $headers
     */
    private function namedDelay(array $headers): ?float
    {
        $written = trim($headers[self::DELAY_HEADER] ?? '');
        if ($written === '' || strlen($written) > self::MAX_DELAY_HEADER_BYTES) {
            return null;
        }
        if (preg_match(self::WHOLE_SECONDS, $written) !== 1) {
            return null;
        }

        $seconds = (int) $written;

        return $seconds > self::MAX_NAMED_DELAY_SECONDS ? null : (float) $seconds;
    }

    /**
     * How long to wait before the next attempt.
     *
     * It is what the API asked for when it asked for anything, and the client's own schedule
     * otherwise. Either way it is cut down to what is left of the budget every delay of one send
     * shares, so a delay written by the other end cannot stretch a send past what the caller allowed
     * for it.
     */
    private function waitFor(Attempt $outcome, float $scheduled, float $remaining): float
    {
        $wanted = $outcome->retryAfter ?? $scheduled;

        return min(max($wanted, 0.0), max($remaining, 0.0));
    }

    /** The identifier the API says it ingested the event under. */
    private function ingestedId(string $payload): ?string
    {
        $answered = $this->parsed($payload);
        if (!is_array($answered) || !isset($answered['event_id'])) {
            return null;
        }

        return is_string($answered['event_id']) ? $answered['event_id'] : null;
    }

    /** The problem a refusal names, unset when the body names none this client can read. */
    private function problemId(string $payload): ?string
    {
        $problem = $this->parsed($payload);
        if (!is_array($problem) || !isset($problem['id'])) {
            return null;
        }

        return is_string($problem['id']) ? $problem['id'] : null;
    }

    private function parsed(string $payload): mixed
    {
        try {
            return Runtime::decodePayload($payload);
        } catch (DecodeError) {
            return null;
        }
    }

    /** What to throw when a send is being given up on. */
    private function givenUp(string $eventId, int $attempts, float $waited, string $detail): ClientError
    {
        if ($attempts <= 1) {
            return ClientError::eventSending($eventId, $detail);
        }

        return ClientError::retriesExhausted($eventId, $attempts, $waited, $detail);
    }

    /**
     * The randomness used to jitter the delays of one send.
     *
     * Jitter only has to keep emitters that failed together from coming back together; it does not
     * have to be unpredictable, so the platform's own generator is enough.
     *
     * @return list<float>
     */
    private function jitterDraws(int $count): array
    {
        $draws = [];
        for ($index = 0; $index < max($count, 0); $index++) {
            $draws[] = mt_rand() / mt_getrandmax();
        }

        return $draws;
    }

    /**
     * The event types an application already declares, out of what the API answered.
     *
     * @return list<string>
     */
    private function declaredEventTypes(): array
    {
        try {
            [$status, $payload] = $this->transport->request(
                'GET',
                self::EVENT_TYPES_PATH,
                [['application_id', $this->applicationId]]
            );
        } catch (TransportError $failure) {
            throw ClientError::availableEventTypes($failure->getMessage());
        }

        if ($status < self::LOWEST_SUCCESS || $status >= self::LOWEST_REDIRECTION) {
            throw ClientError::availableEventTypes($payload);
        }

        $answered = $this->parsed($payload);
        if (!is_array($answered) || (count($answered) > 0 && !array_is_list($answered))) {
            throw ClientError::availableEventTypes('the API did not answer a list of event types');
        }

        $declared = [];
        foreach ($answered as $entry) {
            $name = $this->declaredName($entry);
            if ($name !== null) {
                $declared[] = $name;
            }
        }

        return $declared;
    }

    /** The name one entry of the list the API answered declares, when it declares one. */
    private function declaredName(mixed $entry): ?string
    {
        if (!is_array($entry) || !isset($entry['event_type_name'])) {
            return null;
        }

        return is_string($entry['event_type_name']) ? $entry['event_type_name'] : null;
    }

    /** Declares one event type on the application. */
    private function createEventType(EventType $eventType): void
    {
        $body = [
            'application_id' => $this->applicationId,
            'service' => $eventType->service,
            'resource_type' => $eventType->resourceType,
            'verb' => $eventType->verb,
        ];

        try {
            [$status, $payload] = $this->transport->request(
                'POST',
                self::EVENT_TYPES_PATH,
                [],
                $body
            );
        } catch (TransportError $failure) {
            throw ClientError::creatingEventType((string) $eventType, $failure->getMessage());
        }

        if ($status >= self::LOWEST_SUCCESS && $status < self::LOWEST_REDIRECTION) {
            return;
        }

        throw ClientError::creatingEventType((string) $eventType, $payload);
    }
}

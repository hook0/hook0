<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

use Hook0\Client;
use Hook0\Event;
use Hook0\Options;
use Hook0\RetryPolicy;
use PHPUnit\Framework\TestCase;

/**
 * What every case that talks to an API is built on.
 */
abstract class ApiCase extends TestCase
{
    /** The application and the credential every case is built with. */
    protected const APPLICATION_ID = 'app-123';
    protected const TOKEN = 'token-xyz';

    /** The shape a UUID has, whichever version it carries. */
    protected const UUID_PATTERN = '/^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/';

    protected FakeApi $api;

    protected function setUp(): void
    {
        $this->api = new FakeApi();
    }

    protected function tearDown(): void
    {
        $this->api->close();
    }

    /**
     * One case of the corpus against an API of its own.
     *
     * What is counted is what one send issued, and both a request count and a queue of scripted
     * answers carried over from the case before it would say nothing.
     */
    protected function restarted(): void
    {
        $this->api->close();
        $this->api = new FakeApi();
    }

    /**
     * A schedule short enough that a case spends its time on requests rather than on waiting.
     *
     * Its budget sits far above what its delays add up to, so the number of attempts a case observes
     * is the one its policy asked for rather than the one its budget allowed.
     */
    protected function retries(int $maxAttempts = 4): RetryPolicy
    {
        return new RetryPolicy(
            maxAttempts: $maxAttempts,
            initialBackoff: 0.005,
            maxBackoff: 0.005,
            maxTotalDelay: 1.0
        );
    }

    protected function options(
        int $maxAttempts = 4,
        float $requestTimeout = 5.0,
        ?int $maxPayloadBytes = null,
        ?int $maxResponseBytes = null,
        ?int $maxResponseHeaders = null,
        ?int $maxHeaderBytes = null,
        ?int $maxHeadBytes = null
    ): Options {
        $defaults = new Options();

        return new Options(
            retryPolicy: $this->retries($maxAttempts),
            requestTimeout: $requestTimeout,
            maxPayloadBytes: $maxPayloadBytes ?? $defaults->maxPayloadBytes,
            maxResponseBytes: $maxResponseBytes ?? $defaults->maxResponseBytes,
            maxResponseHeaders: $maxResponseHeaders ?? $defaults->maxResponseHeaders,
            maxHeaderBytes: $maxHeaderBytes ?? $defaults->maxHeaderBytes,
            maxHeadBytes: $maxHeadBytes ?? $defaults->maxHeadBytes
        );
    }

    protected function client(?Options $chosen = null): Client
    {
        return new Client(
            $this->api->baseUrl(),
            self::APPLICATION_ID,
            self::TOKEN,
            $chosen ?? $this->options()
        );
    }

    /**
     * @param array<string, string> $labels
     * @param array<string, string>|null $metadata
     */
    protected function anEvent(
        string $payload = '{"email": "test@example.com"}',
        array $labels = ['environment' => 'production'],
        ?array $metadata = null,
        ?string $eventId = null
    ): Event {
        return new Event(
            eventType: 'auth.user.create',
            payload: $payload,
            payloadContentType: 'application/json',
            labels: $labels,
            metadata: $metadata,
            eventId: $eventId
        );
    }

    protected function ingested(string $eventId, float $heldFor = 0.0): ScriptedResponse
    {
        return new ScriptedResponse(
            201,
            [
                'application_id' => self::APPLICATION_ID,
                'event_id' => $eventId,
                'received_at' => '2026-01-01',
            ],
            $heldFor
        );
    }

    protected function alreadyIngested(): ScriptedResponse
    {
        return new ScriptedResponse(409, [
            'id' => 'EventAlreadyIngested',
            'title' => 'Event already Ingested',
            'detail' => 'This event was previously ingested and recorded inside Hook0 service.',
            'status' => 409,
            'type' => 'https://documentation.hook0.com/problems',
        ]);
    }

    protected function serverError(): ScriptedResponse
    {
        return new ScriptedResponse(500, ['id' => 'InternalServerError', 'status' => 500]);
    }
}

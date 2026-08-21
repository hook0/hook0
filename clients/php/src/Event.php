<?php

declare(strict_types=1);

namespace Hook0;

/**
 * An event to send to Hook0.
 *
 * `$eventId` is the caller's to set when it already has one to key the event on. Left unset, the
 * client generates a UUIDv7, sends it and answers it — which is what lets it repeat a request
 * without risking a second copy of the event being ingested and delivered to every subscriber.
 */
final class Event
{
    /**
     * @param string $eventType the type of the event, as the application declares it
     * @param string $payload what the event carries
     * @param string $payloadContentType how to read the payload
     * @param array<string, string> $labels what the API routes the event by
     * @param array<string, string>|null $metadata anything else worth carrying
     * @param \DateTimeImmutable|null $occurredAt when the event happened; the current moment when unset
     * @param string|null $eventId what to key the event on; the client chooses when unset
     */
    public function __construct(
        public readonly string $eventType,
        public readonly string $payload,
        public readonly string $payloadContentType,
        public readonly array $labels = [],
        public readonly ?array $metadata = null,
        public readonly ?\DateTimeImmutable $occurredAt = null,
        public readonly ?string $eventId = null,
    ) {
    }
}

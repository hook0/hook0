<?php

declare(strict_types=1);

namespace Hook0;

/**
 * An event type, read out of the `service.resource_type.verb` it is written as.
 */
final class EventType implements \Stringable
{
    /** What an event type reads as. */
    private const PATTERN = '/^([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)$/';

    public function __construct(
        public readonly string $service,
        public readonly string $resourceType,
        public readonly string $verb,
    ) {
    }

    /**
     * Reads an event type, refusing one that does not name all three of its parts.
     *
     * @throws ClientError
     */
    public static function parse(string $written): self
    {
        if (preg_match(self::PATTERN, $written, $read) !== 1) {
            throw ClientError::invalidEventType($written);
        }

        return new self($read[1], $read[2], $read[3]);
    }

    /** The event type as the API reads one. */
    public function __toString(): string
    {
        return sprintf('%s.%s.%s', $this->service, $this->resourceType, $this->verb);
    }
}

<?php

declare(strict_types=1);

namespace Hook0;

/**
 * The one failure this client reports.
 *
 * Sending an event, upserting an event type and verifying a webhook all throw this, so a caller has
 * one thing to catch whatever it asked the client to do. The failures the *API* reports are a
 * different matter: those are the problems it names in its own error contract, and the generated
 * half of this package throws one exception per problem.
 *
 * The constructors below are what the client throws through; each one exists so that the same
 * situation always reads the same way, whichever call it came out of.
 */
final class ClientError extends \RuntimeException
{
    /**
     * A send the API refused for a reason repeating it would not change.
     */
    public static function eventSending(string $eventId, string $detail): self
    {
        return new self(sprintf('Sending event %s failed: %s', $eventId, $detail));
    }

    /**
     * A send that ran out of attempts, or out of the delay budget its attempts share.
     *
     * A send that gave up and a single refused request are otherwise indistinguishable to a caller,
     * which is the difference between a transient outage and a request that will never be accepted.
     */
    public static function retriesExhausted(
        string $eventId,
        int $attempts,
        float $waited,
        string $detail
    ): self {
        return new self(sprintf(
            'Sending event %s failed: gave up after %d attempts spread over %ss of retry delay; '
            . 'last failure: %s',
            $eventId,
            $attempts,
            number_format($waited, 3, '.', ''),
            $detail
        ));
    }

    /**
     * A payload above what the client agrees to send, refused before a socket is opened.
     */
    public static function payloadTooLarge(string $eventId, int $size, int $maximum): self
    {
        return new self(sprintf(
            'Sending event %s failed: event payload is %d bytes, which is more than the %d bytes '
            . 'this client sends at most; nothing was sent',
            $eventId,
            $size,
            $maximum
        ));
    }

    /**
     * An event type that does not read as `service.resource_type.verb`.
     */
    public static function invalidEventType(string $eventType): self
    {
        return new self(sprintf(
            "Provided event type '%s' does not have a valid syntax (service.resource_type.verb)",
            $eventType
        ));
    }

    /**
     * The list of event types the application already declares could not be read.
     */
    public static function availableEventTypes(string $detail): self
    {
        return new self(sprintf('Getting available event types failed: %s', $detail));
    }

    /**
     * An event type that could not be created.
     */
    public static function creatingEventType(string $eventType, string $detail): self
    {
        return new self(sprintf("Creating event type '%s' failed: %s", $eventType, $detail));
    }
}

<?php

declare(strict_types=1);

namespace Hook0;

/**
 * What one attempt at sending an event ended with.
 *
 * @internal this is how {@see Client} carries one round trip from where it was read to where it is
 *   decided upon, and nothing outside the client builds or reads one
 */
final class Attempt
{
    /**
     * @param string|null $ingested the identifier the API answered, when it accepted the event
     * @param bool $alreadyIngested whether the API answered that the identifier is already taken
     * @param string $detail what to say about the attempt when it is the last one
     * @param bool $retryable whether repeating the request could end differently
     * @param float|null $retryAfter how long the answer said to wait before repeating the request
     */
    public function __construct(
        public readonly ?string $ingested,
        public readonly bool $alreadyIngested,
        public readonly string $detail,
        public readonly bool $retryable,
        public readonly ?float $retryAfter,
    ) {
    }
}

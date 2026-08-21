<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

/**
 * A request the API received, in the order it received it.
 *
 * The HTTP method is `verb` rather than `method`, so that a case asking what a request was is not
 * asking about a member of some class.
 */
final class ReceivedRequest
{
    /**
     * @param array<string, string> $headers
     */
    public function __construct(
        public readonly string $verb,
        public readonly string $target,
        public readonly array $headers,
        public readonly string $body,
    ) {
    }

    /**
     * What the body carried, read as the JSON document it is.
     *
     * @return array<array-key, mixed>
     */
    public function json(): array
    {
        $read = json_decode($this->body, true, 32, JSON_THROW_ON_ERROR);

        return is_array($read) ? $read : [];
    }
}

<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

/**
 * What the API answers to one request, in the order the case scripted it.
 */
final class ScriptedResponse
{
    /**
     * @param mixed $body what the answer carries, written as JSON unless it is already text
     * @param float $heldFor how long the answer is sat on before it is written, in seconds
     * @param array<string, string> $headers what the answer carries beside its body
     */
    public function __construct(
        public readonly int $status,
        public readonly mixed $body,
        public readonly float $heldFor = 0.0,
        public readonly array $headers = [],
    ) {
    }

    /**
     * @return array{status: int, body: mixed, held_for: float, headers: array<string, string>}
     */
    public function scripted(): array
    {
        return [
            'status' => $this->status,
            'body' => $this->body,
            'held_for' => $this->heldFor,
            'headers' => $this->headers,
        ];
    }
}

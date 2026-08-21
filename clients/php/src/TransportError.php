<?php

declare(strict_types=1);

namespace Hook0;

/**
 * A request the API never answered, and what caused that.
 *
 * The three causes are told apart because only one of them could end differently. A request that
 * got no answer — a connection refused or reset, an attempt out of time, a body that stopped
 * mid-way — says nothing about whether the API acted on it, which is exactly why a send carries an
 * identifier the client chose itself, and why repeating it is safe and worth doing. An answer that
 * crossed a ceiling this client set for itself draws the same answer the second time, and reading it
 * again four times over costs the caller four times as much for the same failure. A URL nothing can
 * be sent to was never sent at all, and a repetition builds the same unusable request, turning a
 * misconfiguration into a message that accuses the network.
 *
 * The names are the ones the shared conformance corpus gives them, so the verdict a client applies
 * and the verdict that corpus writes down are the same words.
 */
final class TransportError extends \RuntimeException
{
    private function __construct(
        string $detail,
        public readonly string $causeName,
        public readonly bool $retryable
    ) {
        parent::__construct($detail);
    }

    /**
     * The API was reached for and answered nothing this client could read to its end.
     */
    public static function noAnswer(string $detail): self
    {
        return new self($detail, 'no_answer', true);
    }

    /**
     * The API answered, and what it answered crossed a ceiling this client set for itself.
     */
    public static function answerAboveABound(string $detail): self
    {
        return new self($detail, 'answer_above_a_bound', false);
    }

    /**
     * There is nowhere to send the request, so nothing was sent.
     */
    public static function unusableApiUrl(string $detail): self
    {
        return new self($detail, 'unusable_api_url', false);
    }
}

<?php

declare(strict_types=1);

namespace Hook0;

/**
 * Every bound a client applies to one send.
 */
final class Options
{
    /**
     * Largest event payload the client agrees to send, in bytes.
     *
     * The API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
     * being refused once the JSON envelope around it — metadata, labels, identifiers — is counted.
     * The client rules such an event out rather than spending a round trip, and every retry after
     * it, on a request that cannot be accepted.
     */
    public const DEFAULT_MAX_PAYLOAD_BYTES = 1024 * 1024;

    /**
     * @param RetryPolicy $retryPolicy how the attempts of one send are spaced out
     * @param float $requestTimeout how long one attempt is given, in seconds
     * @param int $maxPayloadBytes the largest payload sent, refused before a socket is opened
     * @param int $maxResponseBytes the largest answer read off a socket
     * @param int $maxResponseHeaders how many header lines an answer may carry
     * @param int $maxHeaderBytes the longest one header line may be
     * @param int $maxHeadBytes the largest whole head, every line counted together
     */
    public function __construct(
        public readonly RetryPolicy $retryPolicy = new RetryPolicy(),
        public readonly float $requestTimeout = Transport::DEFAULT_REQUEST_TIMEOUT,
        public readonly int $maxPayloadBytes = self::DEFAULT_MAX_PAYLOAD_BYTES,
        public readonly int $maxResponseBytes = Transport::DEFAULT_MAX_RESPONSE_BYTES,
        public readonly int $maxResponseHeaders = Transport::DEFAULT_MAX_RESPONSE_HEADERS,
        public readonly int $maxHeaderBytes = Transport::DEFAULT_MAX_HEADER_BYTES,
        public readonly int $maxHeadBytes = Transport::DEFAULT_MAX_HEAD_BYTES,
    ) {
    }
}

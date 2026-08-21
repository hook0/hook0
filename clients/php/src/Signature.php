<?php

declare(strict_types=1);

namespace Hook0;

/**
 * A signature header, read into the pieces a verification needs, and the verification itself.
 *
 * A signature names the moment it was signed and one or two message authentication codes over the
 * body. The `v1` scheme also covers a list of request headers, so a receiver can tell apart two
 * deliveries that carry the same body but not the same context; `v0` covers the body alone and is
 * what an older sender still produces. When both are offered, `v1` is the one verified: accepting
 * the weaker of two schemes on the strength of the sender offering it is how a downgrade works.
 *
 * Two things are refused before any code is computed. A header the signature says it covers but the
 * request did not carry is refused outright, because signing over an absent value would let a sender
 * drop a header and keep the signature valid. And a signature whose codes are not whole hexadecimal
 * is refused rather than decoded as far as it goes: a decoder that stops at the first bad character
 * compares a prefix, and a prefix of the right code is not the right code.
 */
final class Signature
{
    /**
     * Longest signature header read. The header is written by whoever reached the endpoint, so its
     * size is bounded before any of it is split, decoded or compared.
     */
    public const MAX_SIGNATURE_BYTES = 8 * 1024;

    /** Most `key=value` parts one signature header is split into. */
    public const MAX_SIGNATURE_PARTS = 32;

    /** Most header names one signature covers. */
    public const MAX_COVERED_HEADERS = 64;

    /** Most headers of a delivery read back, whichever way the caller holds them. */
    public const MAX_DELIVERED_HEADERS = 256;

    /**
     * Furthest from the epoch, in either direction, a signature's moment may sit. A header carrying
     * thousands of digits would otherwise reach the arithmetic that holds it against the current
     * time, so the digits are counted before any of them is read as a number.
     */
    public const MAX_TIMESTAMP = 1_000_000_000_000;

    /** How many digits a moment inside that bound is written with, sign excluded. */
    private const MAX_TIMESTAMP_DIGITS = 13;

    /** What separates one part of the signature header from the next. */
    private const PART_SEPARATOR = ',';

    /**
     * What separates the name of a part from its value. Only the first one counts: a value may hold
     * further ones, and splitting on all of them would silently drop everything past the second.
     */
    private const PART_ASSIGNATOR = '=';

    /** What separates two header names inside the `h` part, and what they are joined back with. */
    private const HEADER_NAME_SEPARATOR = ' ';

    /** What separates the pieces of the message a code is computed over. */
    private const MESSAGE_SEPARATOR = '.';

    /** Part naming the moment the delivery was signed, in whole seconds since the Unix epoch. */
    private const TIMESTAMP_PART = 't';

    /** Part carrying the code covering the body alone. */
    private const BODY_SCHEME_PART = 'v0';

    /** Part carrying the code covering the covered headers and the body. */
    private const HEADERS_SCHEME_PART = 'v1';

    /** Part listing the headers the `v1` code covers, in the order it covers them. */
    private const COVERED_HEADERS_PART = 'h';

    /** What a whole number of seconds reads as. */
    private const WHOLE_SECONDS = '/^-?\d+$/';

    /** What a code reads as: whole pairs of hexadecimal digits, and nothing else. */
    private const WHOLE_HEXADECIMAL = '/^(?:[0-9A-Fa-f]{2})+$/';

    /** What a header name is written with, as RFC 9110 spells a token. */
    private const HEADER_NAME = '/^[A-Za-z0-9!#$%&\'*+\-.^_`|~]+$/';

    /** What the codes are computed with. */
    private const DIGEST = 'sha256';

    /**
     * @param int $timestamp the moment the delivery was signed, in whole seconds since the epoch
     * @param list<string> $coveredHeaders the headers the stronger scheme covers, lowercased and in order
     * @param string|null $bodyCode the `v0` code, decoded
     * @param string|null $headersCode the `v1` code, decoded
     */
    private function __construct(
        public readonly int $timestamp,
        public readonly array $coveredHeaders,
        public readonly ?string $bodyCode,
        public readonly ?string $headersCode,
    ) {
    }

    /**
     * Reads a signature header, refusing anything it cannot read whole.
     *
     * @param string $signature the value of the `X-Hook0-Signature` header
     * @throws ClientError for every way a header can fail to be one
     */
    public static function parse(string $signature): self
    {
        if (strlen($signature) > self::MAX_SIGNATURE_BYTES) {
            throw new ClientError(sprintf(
                'the signature is %d characters long, above the %d accepted',
                strlen($signature),
                self::MAX_SIGNATURE_BYTES
            ));
        }

        $read = self::partsOf($signature);
        if (count($read) < 2) {
            throw new ClientError('the signature carries neither a timestamp nor a code');
        }

        $bodyCode = self::codeOf($read, self::BODY_SCHEME_PART);
        $headersCode = self::codeOf($read, self::HEADERS_SCHEME_PART);
        if ($bodyCode === null && $headersCode === null) {
            throw new ClientError(sprintf(
                'the signature carries neither a `%s` nor a `%s` code',
                self::BODY_SCHEME_PART,
                self::HEADERS_SCHEME_PART
            ));
        }

        return new self(
            self::timestampOf($read),
            self::coveredHeadersOf($read),
            $bodyCode,
            $headersCode
        );
    }

    /**
     * Verifies a webhook against a moment the caller names.
     *
     * The clock window is bilateral. A moment too far in the future is refused exactly like one too
     * far in the past, so the window a given delivery is accepted in stays the width the caller asked
     * for, whichever way a clock drifted.
     *
     * @param string $signature the value of the `X-Hook0-Signature` header
     * @param string $payload the raw body of the webhook request
     * @param array<array-key, mixed> $headers the headers of the webhook request, as a map or as pairs
     * @param string $subscriptionSecret the signing secret of the subscription it was delivered for
     * @param float $tolerance how far, in seconds and in either direction, the moment the signature
     *   names may sit from `$currentTime`. Five minutes is a reasonable trade-off between tolerating
     *   clock drift and bounding how long a captured delivery can be replayed.
     * @param \DateTimeImmutable $currentTime what to hold the signature's moment against
     * @throws ClientError for every reason a webhook may be refused
     */
    public static function verifyWithCurrentTime(
        string $signature,
        string $payload,
        array $headers,
        string $subscriptionSecret,
        float $tolerance,
        \DateTimeImmutable $currentTime
    ): void {
        $parsed = self::parse($signature);

        $delivered = self::deliveredHeaders($headers);
        $coveredValues = [];
        foreach ($parsed->coveredHeaders as $name) {
            if (!array_key_exists($name, $delivered)) {
                throw new ClientError(sprintf(
                    'the `%s` header the signature covers was not delivered',
                    $name
                ));
            }
            $coveredValues[] = $delivered[$name];
        }

        if (!$parsed->matches($payload, $coveredValues, $subscriptionSecret)) {
            throw new ClientError('the signature does not match what the subscription secret produces');
        }

        $drift = ((float) $currentTime->format('U.u')) - $parsed->timestamp;
        if (abs($drift) > $tolerance) {
            throw new ClientError(sprintf(
                'the signature was made %s seconds from now, outside the %s accepted',
                number_format($drift, 0, '.', ''),
                self::written($tolerance)
            ));
        }
    }

    /**
     * Verifies a webhook against the current moment.
     *
     * See {@see self::verifyWithCurrentTime()} for what each argument is.
     *
     * @param array<array-key, mixed> $headers
     * @throws ClientError
     */
    public static function verify(
        string $signature,
        string $payload,
        array $headers,
        string $subscriptionSecret,
        float $tolerance
    ): void {
        self::verifyWithCurrentTime(
            $signature,
            $payload,
            $headers,
            $subscriptionSecret,
            $tolerance,
            new \DateTimeImmutable('now')
        );
    }

    /**
     * Whether the code this signature carries is the one the secret produces.
     *
     * The stronger scheme wins when both are offered, and the comparison is made in constant time:
     * one that gave up at the first differing byte would say, by how long it took, how much of a
     * guess was right.
     *
     * @param list<string> $coveredValues the values of the covered headers, in order
     */
    public function matches(string $payload, array $coveredValues, string $subscriptionSecret): bool
    {
        $message = $this->timestamp . self::MESSAGE_SEPARATOR;

        if ($this->headersCode !== null) {
            $message .= implode(self::HEADER_NAME_SEPARATOR, $this->coveredHeaders)
                . self::MESSAGE_SEPARATOR
                . implode(self::MESSAGE_SEPARATOR, $coveredValues)
                . self::MESSAGE_SEPARATOR
                . $payload;

            return hash_equals(
                hash_hmac(self::DIGEST, $message, $subscriptionSecret, true),
                $this->headersCode
            );
        }

        // A signature carrying neither code is refused while it is being read, so what is left here
        // is the body-only scheme.
        $message .= $payload;

        return hash_equals(
            hash_hmac(self::DIGEST, $message, $subscriptionSecret, true),
            (string) $this->bodyCode
        );
    }

    /**
     * The `key=value` parts of a header, split on the first assignator of each and trimmed.
     *
     * @return array<string, string>
     */
    private static function partsOf(string $signature): array
    {
        $parts = explode(self::PART_SEPARATOR, $signature);
        if (count($parts) > self::MAX_SIGNATURE_PARTS) {
            throw new ClientError(sprintf(
                'the signature carries more than the %d parts accepted',
                self::MAX_SIGNATURE_PARTS
            ));
        }

        $read = [];
        foreach ($parts as $part) {
            $name = strstr($part, self::PART_ASSIGNATOR, true);
            if ($name === false) {
                continue;
            }
            $value = substr($part, strlen($name) + strlen(self::PART_ASSIGNATOR));
            $read[trim($name)] = trim($value);
        }

        return $read;
    }

    /**
     * The moment the signature names, which it is not a signature without.
     *
     * @param array<string, string> $read
     */
    private static function timestampOf(array $read): int
    {
        if (!isset($read[self::TIMESTAMP_PART])) {
            throw new ClientError(
                sprintf('the signature carries no `%s` part', self::TIMESTAMP_PART)
            );
        }

        $written = $read[self::TIMESTAMP_PART];
        if (preg_match(self::WHOLE_SECONDS, $written) !== 1) {
            throw new ClientError(sprintf('`%s` is not a number of seconds', $written));
        }
        if (strlen(ltrim($written, '-')) > self::MAX_TIMESTAMP_DIGITS) {
            throw new ClientError(sprintf(
                "the signature's moment is further than %d seconds from the epoch",
                self::MAX_TIMESTAMP
            ));
        }

        $seconds = (int) $written;
        if (abs($seconds) > self::MAX_TIMESTAMP) {
            throw new ClientError(sprintf(
                "the signature's moment is further than %d seconds from the epoch",
                self::MAX_TIMESTAMP
            ));
        }

        return $seconds;
    }

    /**
     * One of the codes a signature offers, decoded whole or not at all.
     *
     * The shape is checked before anything is decoded, so the decoder is never handed an odd number
     * of digits or a character it cannot read — both of which it would answer with a warning and a
     * false, and a false read as a code is a comparison against nothing.
     *
     * @param array<string, string> $read
     */
    private static function codeOf(array $read, string $part): ?string
    {
        if (!isset($read[$part])) {
            return null;
        }

        $written = $read[$part];
        if (preg_match(self::WHOLE_HEXADECIMAL, $written) !== 1) {
            throw new ClientError(sprintf('the `%s` code is not hexadecimal', $part));
        }

        return (string) hex2bin($written);
    }

    /**
     * The headers the stronger scheme covers, in the order it covers them.
     *
     * @param array<string, string> $read
     * @return list<string>
     */
    private static function coveredHeadersOf(array $read): array
    {
        $written = $read[self::COVERED_HEADERS_PART] ?? '';
        if ($written === '') {
            return [];
        }

        $names = explode(self::HEADER_NAME_SEPARATOR, $written);
        if (count($names) > self::MAX_COVERED_HEADERS) {
            throw new ClientError(sprintf(
                'the signature covers more than the %d headers accepted',
                self::MAX_COVERED_HEADERS
            ));
        }

        $covered = [];
        foreach ($names as $name) {
            if (preg_match(self::HEADER_NAME, $name) !== 1) {
                throw new ClientError(sprintf('`%s` is not a header name', $name));
            }
            $covered[] = strtolower($name);
        }

        return $covered;
    }

    /**
     * The headers of the request, under the names a signature refers to them by.
     *
     * A caller holds them either as a map of name to value or as a list of pairs, and both read the
     * same way here. A later value wins over an earlier one under the same name, which is what a map
     * built by the caller would have done.
     *
     * @param array<array-key, mixed> $headers
     * @return array<string, string>
     */
    private static function deliveredHeaders(array $headers): array
    {
        if (count($headers) > self::MAX_DELIVERED_HEADERS) {
            throw new ClientError(sprintf(
                'the delivery carries more than the %d headers accepted',
                self::MAX_DELIVERED_HEADERS
            ));
        }

        $delivered = [];
        foreach ($headers as $key => $carried) {
            if (is_array($carried)) {
                $pair = array_values($carried);
                if (count($pair) !== 2) {
                    throw new ClientError('a header is not a name and a value');
                }
                [$name, $value] = $pair;
            } else {
                $name = $key;
                $value = $carried;
            }

            $delivered[strtolower(self::headerText($name))] = self::headerText($value);
        }

        return $delivered;
    }

    /** A header name or value as text, whichever way the caller holds it. */
    private static function headerText(mixed $value): string
    {
        if (!is_string($value)) {
            throw new ClientError(
                sprintf('a header is %s, not a header value', get_debug_type($value))
            );
        }
        if (preg_match('//u', $value) !== 1) {
            throw new ClientError('a header is not UTF-8');
        }

        return $value;
    }

    /** A number of seconds, written the way a message reads it back. */
    private static function written(float $seconds): string
    {
        return $seconds === floor($seconds)
            ? number_format($seconds, 1, '.', '')
            : (string) $seconds;
    }
}

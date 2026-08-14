<?php

declare(strict_types=1);

namespace Hook0;

/**
 * What the generated half of this package reads and writes values through.
 *
 * Everything here is hand-written and never regenerated. It is the one seam between what the API
 * declares — the classes, the enumerations, the problems and the methods the generator writes under
 * `src/Generated/` — and what it does not: how a JSON document is turned into a value, and what
 * happens to a document that does not say what it was declared to say.
 *
 * Reading is deliberately strict. A member the document declares as a string and the API answered as
 * a number stops the read with the name of that member, rather than yielding an object whose
 * declared type lies about what it holds. Every failure of that kind is a {@see DecodeError}, so a
 * caller has one thing to catch whatever the shape of the answer was. That strictness is also what
 * makes the generated constructors safe: a promoted property is typed, and the value handed to it
 * has already been held to that type here.
 *
 * A reader is a closure taking what the document carried and answering the value it stands for. The
 * scalar ones are methods, reached as first-class callables; the ones built around another reader
 * answer a closure of their own.
 */
final class Runtime
{
    /**
     * Longest fragment of a response body an error message carries. Bodies are answered by a server
     * this package does not control, so they are cut at a fixed budget rather than echoed whole into
     * whatever the caller logs.
     */
    public const MAX_PREVIEW_BYTES = 256;

    /**
     * Largest JSON document read out of a response body, in bytes. The transport caps what it reads
     * off a socket; this caps what is handed to the parser whichever way the bytes arrived.
     */
    public const MAX_PAYLOAD_BYTES = 8 * 1024 * 1024;

    /**
     * Deepest a JSON document may nest before the parser gives up, which is what keeps a document
     * that is nothing but brackets from growing the stack.
     */
    public const MAX_PAYLOAD_NESTING = 64;

    /** The shape a UUID is written in, whichever version it carries. */
    private const UUID_PATTERN = '/^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}$/';

    /** A moment, as RFC 3339 spells one: a day, a time, an optional fraction, and an offset. */
    private const DATE_TIME_PATTERN = '/^(\d{4}-\d{2}-\d{2})T\d{2}:\d{2}:\d{2}(\.\d+)?([Zz]|[+-]\d{2}:\d{2})$/';

    /** A day, as ISO 8601 spells one. */
    private const DATE_PATTERN = '/^\d{4}-\d{2}-\d{2}$/';

    /** How a moment is written back when it carries no fraction of a second, and when it does. */
    private const MOMENT_FORMAT = 'Y-m-d\TH:i:sP';
    private const PRECISE_MOMENT_FORMAT = 'Y-m-d\TH:i:s.uP';

    /** How a day is written back. */
    private const DAY_FORMAT = 'Y-m-d';

    /** A string, refusing what merely spells like one. */
    public static function text(mixed $value): string
    {
        if (!is_string($value)) {
            throw new DecodeError(sprintf('expected a string, got %s', self::kind($value)));
        }

        return $value;
    }

    /**
     * A UUID, as the document spells one. It travels as the text the API answered, since that text
     * is what has to go back out unchanged.
     */
    public static function uuid(mixed $value): string
    {
        $text = self::text($value);
        if (preg_match(self::UUID_PATTERN, $text) !== 1) {
            throw new DecodeError(sprintf('expected a UUID, got `%s`', $text));
        }

        return $text;
    }

    /** A whole number. `true` is not one, here or on the wire. */
    public static function integer(mixed $value): int
    {
        if (!is_int($value)) {
            throw new DecodeError(sprintf('expected a whole number, got %s', self::kind($value)));
        }

        return $value;
    }

    /** A number, whether the document wrote it with a fractional part or not. */
    public static function float(mixed $value): float
    {
        if (!is_int($value) && !is_float($value)) {
            throw new DecodeError(sprintf('expected a number, got %s', self::kind($value)));
        }

        return (float) $value;
    }

    /** A boolean, refusing the numbers that stand in for one elsewhere. */
    public static function boolean(mixed $value): bool
    {
        if (!is_bool($value)) {
            throw new DecodeError(sprintf('expected a boolean, got %s', self::kind($value)));
        }

        return $value;
    }

    /**
     * A moment, as RFC 3339 spells one.
     *
     * The shape is checked before the value is built, and the day the value carries is held against
     * the day that was written: the language rolls `2026-02-31` forward to March rather than
     * refusing it, and a moment that quietly moved is worse than one that did not read.
     */
    public static function dateTime(mixed $value): \DateTimeImmutable
    {
        $text = self::text($value);
        if (preg_match(self::DATE_TIME_PATTERN, $text, $read) !== 1) {
            throw new DecodeError(sprintf('expected a date and a time, got `%s`', $text));
        }

        $moment = self::built($text);
        if ($moment->format(self::DAY_FORMAT) !== $read[1]) {
            throw new DecodeError(sprintf('`%s` names a day that does not exist', $text));
        }

        return $moment;
    }

    /**
     * A day, as ISO 8601 spells one.
     *
     * The moment it carries is the start of that day in UTC, since the document says nothing about
     * a time and a value read at whatever the machine's zone happens to be would not round-trip.
     */
    public static function date(mixed $value): \DateTimeImmutable
    {
        $text = self::text($value);
        if (preg_match(self::DATE_PATTERN, $text) !== 1) {
            throw new DecodeError(sprintf('expected a date, got `%s`', $text));
        }

        $day = self::built($text . 'T00:00:00+00:00');
        if ($day->format(self::DAY_FORMAT) !== $text) {
            throw new DecodeError(sprintf('`%s` names a day that does not exist', $text));
        }

        return $day;
    }

    /** A value the document does not describe, which is therefore kept as it arrived. */
    public static function jsonValue(mixed $value): mixed
    {
        return $value;
    }

    /**
     * Every item of an array, each one read the same way.
     */
    public static function listOf(\Closure $reader): \Closure
    {
        return static function (mixed $value) use ($reader): array {
            if (!is_array($value) || (count($value) > 0 && !array_is_list($value))) {
                throw new DecodeError(sprintf('expected an array, got %s', self::kind($value)));
            }

            return array_map($reader, $value);
        };
    }

    /**
     * Every value of an object whose keys the document leaves open.
     *
     * An open-keyed object carrying nothing is written back as an object rather than as an empty
     * array, so a value read out of what this client wrote is the one or the other; both are read
     * here, and everything else is refused.
     */
    public static function mapOf(\Closure $reader): \Closure
    {
        return static function (mixed $value) use ($reader): array {
            $entries = self::entriesOf($value);
            if ($entries === null) {
                throw new DecodeError(sprintf('expected an object, got %s', self::kind($value)));
            }

            $read = [];
            foreach ($entries as $key => $item) {
                $read[self::text((string) $key)] = $reader($item);
            }

            return $read;
        };
    }

    /**
     * One of the values a closed list declares, refusing anything the list does not carry.
     *
     * @param class-string<\BackedEnum> $declared the enumeration the generator wrote for that list
     */
    public static function memberOf(string $declared): \Closure
    {
        return static function (mixed $value) use ($declared): \BackedEnum {
            $text = self::text($value);
            $member = $declared::tryFrom($text);
            if ($member === null) {
                throw new DecodeError(
                    sprintf('`%s` is not one of the values %s declares', $text, $declared)
                );
            }

            return $member;
        };
    }

    /**
     * The members of an object the document declares, under the name it declares it with.
     *
     * @return array<array-key, mixed>
     */
    public static function asFields(mixed $value, string $owner): array
    {
        $entries = self::entriesOf($value);
        if ($entries === null) {
            throw new DecodeError(sprintf('%s is not a JSON object', $owner));
        }

        return $entries;
    }

    /**
     * A member the document requires, which is therefore missing when it is absent.
     *
     * @param array<array-key, mixed> $fields
     */
    public static function read(array $fields, string $key, \Closure $reader): mixed
    {
        if (!array_key_exists($key, $fields)) {
            throw new DecodeError(sprintf('`%s` is required and was not answered', $key));
        }

        return self::named($key, $reader, $fields[$key]);
    }

    /**
     * A member the document does not require, absent as readily as answered as null.
     *
     * @param array<array-key, mixed> $fields
     */
    public static function maybe(array $fields, string $key, \Closure $reader): mixed
    {
        if (!isset($fields[$key])) {
            return null;
        }

        return self::named($key, $reader, $fields[$key]);
    }

    /**
     * The JSON document a response body carries.
     */
    public static function decodePayload(string $payload): mixed
    {
        if (strlen($payload) > self::MAX_PAYLOAD_BYTES) {
            throw new DecodeError(sprintf(
                'the response is %d bytes, above the %d accepted',
                strlen($payload),
                self::MAX_PAYLOAD_BYTES
            ));
        }

        try {
            return json_decode($payload, true, self::MAX_PAYLOAD_NESTING, JSON_THROW_ON_ERROR);
        } catch (\JsonException $failure) {
            throw new DecodeError(sprintf(
                'the response is not JSON: %s (%s)',
                self::preview($payload),
                $failure->getMessage()
            ));
        }
    }

    /**
     * A value as the API reads it.
     *
     * Slashes and non-ASCII characters are left as they are: what travels is JSON either way, and
     * escaping them would make two documents carrying the same value compare as two.
     */
    public static function encode(mixed $value): string
    {
        try {
            return json_encode(
                $value,
                JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE,
                self::MAX_PAYLOAD_NESTING
            );
        } catch (\JsonException $failure) {
            throw new DecodeError(
                sprintf('the value cannot be written as JSON: %s', $failure->getMessage())
            );
        }
    }

    /**
     * A moment, written the way the API reads one.
     *
     * A moment carrying no fraction of a second is written without one, and one that does keeps the
     * digits it has, so that what was read comes back out unchanged either way.
     */
    public static function moment(\DateTimeImmutable $moment): string
    {
        $format = $moment->format('u') === '000000'
            ? self::MOMENT_FORMAT
            : self::PRECISE_MOMENT_FORMAT;

        return $moment->format($format);
    }

    /** A day, written the way the API reads one. */
    public static function day(\DateTimeImmutable $day): string
    {
        return $day->format(self::DAY_FORMAT);
    }

    /**
     * An open-keyed object, written as one whether or not it carries anything.
     *
     * The language spells a list and a map with the one array, so an empty map would go out as `[]`
     * and be refused by an API expecting an object. This is where the two are told apart, once,
     * rather than at every place a map is written.
     *
     * @param array<array-key, mixed> $values
     * @return array<array-key, mixed>|\stdClass
     */
    public static function mapping(array $values): array|\stdClass
    {
        return count($values) === 0 ? new \stdClass() : $values;
    }

    /**
     * Where a request lands, with each placeholder of the template filled in.
     *
     * @param array<string, mixed> $filled the value each placeholder carries
     */
    public static function path(string $template, array $filled = []): string
    {
        $written = $template;
        foreach ($filled as $name => $value) {
            $written = str_replace('{' . $name . '}', self::pathSegment($value), $written);
        }

        return $written;
    }

    /** A value as one segment of a path, with nothing left in it that could name another one. */
    public static function pathSegment(mixed $value): string
    {
        return rawurlencode(self::written($value));
    }

    /**
     * What travels in the query string: everything the document requires, and everything it does not
     * that the caller actually passed.
     *
     * @param list<array{0: string, 1: mixed}> $required pairs the operation always sends
     * @param list<array{0: string, 1: mixed}> $optional pairs it sends only when they carry something
     * @return list<array{0: string, 1: string}>
     */
    public static function query(array $required, array $optional = []): array
    {
        $pairs = [];
        foreach ($required as [$name, $value]) {
            $pairs[] = [$name, self::written($value)];
        }
        foreach ($optional as [$name, $value]) {
            if ($value !== null) {
                $pairs[] = [$name, self::written($value)];
            }
        }

        return $pairs;
    }

    /** How a value travels in a request line, which is not always how the language prints it. */
    public static function written(mixed $value): string
    {
        return match (true) {
            $value === true => 'true',
            $value === false => 'false',
            $value instanceof \DateTimeImmutable => self::moment($value),
            $value instanceof \BackedEnum => (string) $value->value,
            is_string($value) => $value,
            is_int($value) || is_float($value) => (string) $value,
            default => throw new DecodeError(
                sprintf('a %s cannot travel in a request line', self::kind($value))
            ),
        };
    }

    /**
     * As much of a response body as a message may carry.
     *
     * The cut lands on a byte boundary, and the body is written by a server this client does not
     * control, so what is left is held to being text: a preview that no longer reads as UTF-8 has
     * its non-ASCII bytes replaced rather than travelling whole into whatever the caller logs.
     */
    public static function preview(string $payload): string
    {
        $kept = substr($payload, 0, self::MAX_PREVIEW_BYTES);
        if (preg_match('//u', $kept) !== 1) {
            $kept = (string) preg_replace('/[\x80-\xFF]/', '?', $kept);
        }

        return strlen($payload) > self::MAX_PREVIEW_BYTES ? $kept . '…' : $kept;
    }

    /** What to say about an answer the API document does not describe. */
    public static function unreadable(int $status, string $payload): string
    {
        return sprintf(
            'the API answered %d with a body this client cannot read: %s',
            $status,
            self::preview($payload)
        );
    }

    /**
     * What to say about a problem the API reported.
     *
     * @param object $problem the problem document the API answered
     */
    public static function reported(int $status, object $problem): string
    {
        $written = method_exists($problem, 'toArray') ? self::encode($problem->toArray()) : '';

        return sprintf('the API answered %d: %s', $status, $written);
    }

    /**
     * The members of a JSON object, whichever of the two shapes one arrived as.
     *
     * @return array<array-key, mixed>|null
     */
    private static function entriesOf(mixed $value): ?array
    {
        if ($value instanceof \stdClass) {
            return get_object_vars($value);
        }
        if (is_array($value) && (count($value) === 0 || !array_is_list($value))) {
            return $value;
        }

        return null;
    }

    /** Reads a member, saying which member it was that could not be read. */
    private static function named(string $key, \Closure $reader, mixed $value): mixed
    {
        try {
            return $reader($value);
        } catch (DecodeError $failure) {
            throw new DecodeError(sprintf('`%s`: %s', $key, $failure->getMessage()));
        }
    }

    /** A moment the shape of which has already been checked. */
    private static function built(string $text): \DateTimeImmutable
    {
        try {
            return new \DateTimeImmutable($text);
        } catch (\Exception $failure) {
            throw new DecodeError(
                sprintf('expected a date and a time, got `%s`: %s', $text, $failure->getMessage())
            );
        }
    }

    /** What a value is, as a message names it. */
    private static function kind(mixed $value): string
    {
        return get_debug_type($value);
    }
}

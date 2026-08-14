<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

/**
 * The shared contract every SDK is held to, read rather than transcribed.
 *
 * The corpus sits at `clients/conformance`, is hand-authored, and is read by the suite of every SDK.
 * Nothing in this package writes down a verdict, a bound or a signature of its own: they are read out
 * of the committed documents. A case added to the corpus is therefore exercised here without a line
 * of PHP being edited, and a verdict changed there fails here until this client agrees with it again.
 *
 * The counter-examples kept beside the properties are read the same way: one JSON value per line, so
 * that a header carrying a comma, a newline or nothing at all is read back exactly as written down.
 */
final class Contract
{
    /**
     * Largest document read back. Both corpora are committed, so one above this is one that grew out
     * of shape rather than one somebody meant.
     */
    public const MAX_CORPUS_BYTES = 512 * 1024;

    /** Most counter-examples one property is held against. */
    public const MAX_REGRESSIONS = 1024;

    /**
     * One document of the shared contract, bounded before it is parsed.
     *
     * @return array<string, mixed>
     */
    public static function of(string $name): array
    {
        $path = __DIR__ . '/../../../conformance/' . $name;

        return self::read($path);
    }

    /**
     * The counter-examples worth keeping, committed beside the properties they broke.
     *
     * @return list<mixed>
     */
    public static function regressions(string $name): array
    {
        $path = __DIR__ . '/../regressions/' . $name . '.jsonl';
        $written = self::bounded($path);

        $lines = array_filter(
            explode("\n", $written),
            static fn (string $line): bool => trim($line) !== ''
        );
        if (count($lines) > self::MAX_REGRESSIONS) {
            throw new \RuntimeException(sprintf(
                '%s carries more than the %d counter-examples read back',
                $path,
                self::MAX_REGRESSIONS
            ));
        }

        return array_values(array_map(
            static fn (string $line): mixed => json_decode($line, true, 32, JSON_THROW_ON_ERROR),
            $lines
        ));
    }

    /**
     * @return array<string, mixed>
     */
    private static function read(string $path): array
    {
        $decoded = json_decode(self::bounded($path), true, 32, JSON_THROW_ON_ERROR);

        return is_array($decoded) ? $decoded : [];
    }

    private static function bounded(string $path): string
    {
        $size = filesize($path);
        if ($size === false) {
            throw new \RuntimeException(sprintf('%s cannot be read', $path));
        }
        if ($size > self::MAX_CORPUS_BYTES) {
            throw new \RuntimeException(sprintf(
                '%s is %d bytes long, above the %d read back',
                $path,
                $size,
                self::MAX_CORPUS_BYTES
            ));
        }

        return (string) file_get_contents($path);
    }
}

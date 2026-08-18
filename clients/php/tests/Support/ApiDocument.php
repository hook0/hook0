<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

/**
 * What the API declares, read out of the document the generator was run against.
 *
 * Nothing here names an operation. The document is the one place that says which requests exist, so
 * it is what the suite holds the generated half to: an operation the API grows is one more entry
 * here the moment the snapshot carries it, and one it drops takes its entry with it.
 */
final class ApiDocument
{
    /**
     * The tag that marks an operation as part of the surface an SDK exposes, which is the rule the
     * generator applies — see `PUBLIC_TAG` in `clients/sdkgen/src/snapshot.rs`.
     */
    private const PUBLIC_TAG = 'public';

    /** The methods a request line can carry, which is what tells an operation from the rest. */
    private const VERBS = ['get', 'put', 'post', 'delete', 'options', 'head', 'patch', 'trace'];

    /** No checkout nests this package deeper than this below its root. */
    private const MAX_ANCESTORS = 8;

    /** Largest document read back, far above what the API's snapshot ever is. */
    private const MAX_DOCUMENT_BYTES = 8 * 1024 * 1024;

    /** @var list<DeclaredOperation>|null */
    private static ?array $operations = null;

    /**
     * Every operation an SDK is built out of.
     *
     * A document that marks nothing public exposes all of itself, and one that marks anything
     * exposes what it marked. Both are what the generator does with the tag.
     *
     * @return list<DeclaredOperation>
     */
    public static function operations(): array
    {
        if (self::$operations !== null) {
            return self::$operations;
        }

        $document = self::read();
        $found = [];
        foreach ($document['paths'] as $template => $item) {
            foreach ($item as $verb => $operation) {
                if (!in_array($verb, self::VERBS, true)) {
                    continue;
                }
                $found[] = [
                    in_array(self::PUBLIC_TAG, $operation['tags'] ?? [], true),
                    self::declared((string) $template, $verb, $operation),
                ];
            }
        }
        if ($found === []) {
            throw new \RuntimeException('the API document declares no operation at all');
        }

        $public = array_values(array_filter($found, static fn (array $entry): bool => $entry[0]));
        $kept = $public === [] ? $found : $public;

        self::$operations = array_map(
            static fn (array $entry): DeclaredOperation => $entry[1],
            $kept
        );

        return self::$operations;
    }

    /**
     * Which operation of the document a request landed on, refusing a request that is none or many.
     */
    public static function reached(ReceivedRequest $request): DeclaredOperation
    {
        $matched = array_values(array_filter(
            self::operations(),
            static fn (DeclaredOperation $operation): bool => $operation->matches($request)
        ));
        if (count($matched) !== 1) {
            throw new \RuntimeException(sprintf(
                '`%s %s` is %d of the operations the document declares',
                $request->verb,
                $request->target,
                count($matched)
            ));
        }

        return $matched[0];
    }

    /**
     * @param array<string, mixed> $operation
     */
    private static function declared(string $template, string $verb, array $operation): DeclaredOperation
    {
        /** @var list<array<string, mixed>> $parameters */
        $parameters = $operation['parameters'] ?? [];
        $query = array_values(array_filter(
            $parameters,
            static fn (array $parameter): bool => ($parameter['in'] ?? '') === 'query'
        ));

        $required = [];
        $optional = [];
        foreach ($query as $parameter) {
            $name = (string) $parameter['name'];
            if (($parameter['required'] ?? false) === true) {
                $required[] = $name;
            } else {
                $optional[] = $name;
            }
        }
        sort($required);
        sort($optional);

        return new DeclaredOperation(strtoupper($verb), $template, $required, $optional);
    }

    /**
     * The OpenAPI document the generator was run against, out of the repository holding it.
     *
     * @return array{paths: array<string, array<string, mixed>>}
     */
    private static function read(): array
    {
        $at = __DIR__;
        for ($walked = 0; $walked < self::MAX_ANCESTORS; $walked++) {
            $candidate = $at . '/api/openapi.snapshot.json';
            if (is_file($candidate)) {
                $body = (string) file_get_contents($candidate);
                if (strlen($body) > self::MAX_DOCUMENT_BYTES) {
                    throw new \RuntimeException(sprintf('%s is larger than this suite reads', $candidate));
                }

                /** @var array{paths: array<string, array<string, mixed>>} $document */
                $document = json_decode($body, true, 64, JSON_THROW_ON_ERROR);

                return $document;
            }
            $at = dirname($at);
        }

        throw new \RuntimeException(sprintf(
            'no `api/openapi.snapshot.json` within %d directories of %s',
            self::MAX_ANCESTORS,
            __DIR__
        ));
    }
}

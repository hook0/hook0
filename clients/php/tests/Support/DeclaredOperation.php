<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

/**
 * One operation the API document declares, as a request has to look to be it.
 */
final class DeclaredOperation
{
    /**
     * @param string $template the path with its parameters still written `{like_this}`
     * @param list<string> $requiredQuery names the operation always carries in its query
     * @param list<string> $optionalQuery names it carries only when it was asked with them
     */
    public function __construct(
        public readonly string $verb,
        public readonly string $template,
        public readonly array $requiredQuery,
        public readonly array $optionalQuery,
    ) {
    }

    /** How this operation reads in a message, which is what names it when one fails. */
    public function named(): string
    {
        return $this->verb . ' ' . $this->template;
    }

    /** Every name the query may carry, whether the operation requires it or not. */
    public function queryNames(bool $withOptional): array
    {
        $names = $withOptional
            ? array_merge($this->requiredQuery, $this->optionalQuery)
            : $this->requiredQuery;
        sort($names);

        return $names;
    }

    /** Whether a request landed on this operation. */
    public function matches(ReceivedRequest $request): bool
    {
        if ($request->verb !== $this->verb) {
            return false;
        }

        $wanted = explode('/', $this->template);
        $sent = explode('/', self::pathOf($request->target));
        if (count($wanted) !== count($sent)) {
            return false;
        }

        foreach ($wanted as $index => $declared) {
            if (str_starts_with($declared, '{') && str_ends_with($declared, '}')) {
                // A parameter stands for a segment that is there; an empty one is the trailing
                // slash of another path rather than a value.
                if ($sent[$index] === '') {
                    return false;
                }
                continue;
            }
            if ($declared !== $sent[$index]) {
                return false;
            }
        }

        return true;
    }

    /** The segments of the path a request landed on, in the order the template writes them. */
    public function segmentsOf(ReceivedRequest $request): array
    {
        return explode('/', self::pathOf($request->target));
    }

    /**
     * What the query of a request carried, by name.
     *
     * @return array<string, list<string>>
     */
    public static function queryOf(ReceivedRequest $request): array
    {
        $written = strstr($request->target, '?');
        if ($written === false) {
            return [];
        }

        $carried = [];
        foreach (explode('&', substr($written, 1)) as $pair) {
            if ($pair === '') {
                continue;
            }
            [$name, $value] = array_pad(explode('=', $pair, 2), 2, '');
            $carried[rawurldecode($name)][] = rawurldecode($value);
        }

        return $carried;
    }

    private static function pathOf(string $target): string
    {
        $written = strstr($target, '?', true);

        return $written === false ? $target : $written;
    }
}

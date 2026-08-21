<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

/**
 * One value of whatever the generated half declares, built out of what it declares it as.
 *
 * The language spells a UUID, a moment and a list of names all as one type, so what tells them
 * apart is what the generator wrote beside them: the docblock of a constructor names the type of
 * every member, and the docblock of an operation names both the type of every argument and the
 * wire name it travels under. Nothing here names a schema, a member or an operation — a value of
 * anything the generator writes is built by reading what it wrote about it.
 */
final class Synthesis
{
    /**
     * What every string-shaped member of a value is given.
     *
     * A UUID, because the language spells a UUID as a string and the reader for one refuses
     * anything else: a member that turns out to be a UUID round-trips, and one that is free text
     * carries a UUID through unchanged.
     */
    public const MODEL_TEXT = '3f2504e0-4f89-41d3-9a0c-0305e82c3301';

    /**
     * What every string-shaped argument of an operation is given.
     *
     * It carries the two characters a path segment may not leave as they are, so a value reaching
     * a path proves it was escaped rather than pasted.
     */
    public const ARGUMENT_TEXT = 'a value/with a space';

    /** The moment every date-shaped member carries, which is midnight so that a day round-trips too. */
    public const A_MOMENT = '2026-01-02T00:00:00+00:00';

    /** What a member the document describes nothing about carries, kept as it arrived. */
    public const AN_OPAQUE_VALUE = ['the document' => ['describes', 'none of this']];

    /** No schema the API declares nests anywhere near this deep. */
    private const MAX_DEPTH = 8;

    /** One value of a schema the API declares, with every member it may leave out set or not. */
    public static function model(string $declared, bool $optionals): object
    {
        return self::built($declared, $optionals, 0);
    }

    /**
     * What one operation is asked with: everything it requires, and what it does not as asked for.
     *
     * @return array<string, mixed>
     */
    public static function arguments(\ReflectionMethod $operation, bool $optionals): array
    {
        $types = self::declaredTypes($operation);
        $given = [];
        foreach ($operation->getParameters() as $parameter) {
            $name = $parameter->getName();
            if ($parameter->isDefaultValueAvailable() && !$optionals) {
                continue;
            }
            $given[$name] = self::valueOf(self::peeled($types[$name]), $optionals, self::ARGUMENT_TEXT, 0);
        }

        return $given;
    }

    /**
     * The type the generator wrote for every argument or member of what it declared, by name.
     *
     * @return array<string, string>
     */
    public static function declaredTypes(\ReflectionFunctionAbstract $declared): array
    {
        $written = $declared->getDocComment();
        if ($written === false) {
            return [];
        }

        preg_match_all('/@param\s+(\S.*?)\s+\$(\w+)/', $written, $found, PREG_SET_ORDER);
        $types = [];
        foreach ($found as $entry) {
            $types[$entry[2]] = $entry[1];
        }

        return $types;
    }

    /**
     * The wire name every argument or member travels under, by the name the language calls it.
     *
     * @return array<string, string>
     */
    public static function wireNames(\ReflectionFunctionAbstract $declared): array
    {
        $written = $declared->getDocComment();
        if ($written === false) {
            return [];
        }

        preg_match_all('/@param\s+\S.*?\s+\$(\w+)\s+carries\s+`([^`]+)`/', $written, $found, PREG_SET_ORDER);
        $names = [];
        foreach ($found as $entry) {
            $names[$entry[1]] = $entry[2];
        }

        return $names;
    }

    /**
     * What a method answers, as the generator wrote it: the type of the value, and whether the
     * operation answers a list of them rather than one.
     *
     * @return array{0: string, 1: bool}|null nothing at all when the operation answers nothing
     */
    public static function answered(\ReflectionMethod $operation): ?array
    {
        $returned = $operation->getReturnType();
        if (!$returned instanceof \ReflectionNamedType || $returned->getName() === 'void') {
            return null;
        }
        if ($returned->getName() !== 'array') {
            return [$returned->getName(), false];
        }

        $written = (string) $operation->getDocComment();
        if (preg_match('/@return\s+list<([^>]+)>/', $written, $found) !== 1) {
            throw new \RuntimeException(sprintf(
                '%s answers an array the generator did not say the shape of',
                $operation->getName()
            ));
        }

        return [trim($found[1]), true];
    }

    /** One value of the type the generator wrote, as a member of a value carries it. */
    public static function valueFor(string $type, bool $optionals): mixed
    {
        return self::valueOf($type, $optionals, self::MODEL_TEXT, 0);
    }

    /** How a value the API answered is written back into the document it came out of. */
    public static function written(mixed $value): mixed
    {
        if (is_array($value)) {
            return array_map(self::written(...), $value);
        }
        if (is_object($value) && method_exists($value, 'toArray')) {
            return $value->toArray();
        }

        return $value;
    }

    /** One value of the type a member or an argument declares. */
    private static function valueOf(string $type, bool $optionals, string $text, int $depth): mixed
    {
        if ($depth > self::MAX_DEPTH) {
            throw new \RuntimeException(sprintf('`%s` nests more than %d deep', $type, self::MAX_DEPTH));
        }

        if (preg_match('/^list<(.+)>$/', $type, $found) === 1) {
            return [self::valueOf(trim($found[1]), $optionals, $text, $depth + 1)];
        }
        if (preg_match('/^array<[^,]+,\s*(.+)>$/', $type, $found) === 1) {
            return ['a key' => self::valueOf(trim($found[1]), $optionals, $text, $depth + 1)];
        }

        return match ($type) {
            'mixed' => self::AN_OPAQUE_VALUE,
            'string' => $text,
            'int' => 12,
            'bool' => true,
            'float' => 1.5,
            '\DateTimeImmutable' => new \DateTimeImmutable(self::A_MOMENT),
            default => self::declaredValue(self::resolved($type), $optionals, $depth),
        };
    }

    /** One value of something the generator declared: a closed list of strings, or a schema. */
    private static function declaredValue(string $declared, bool $optionals, int $depth): object
    {
        if (enum_exists($declared)) {
            // Which value of the list it is does not matter; that it is one of them does.
            $cases = $declared::cases();
            if ($cases === []) {
                throw new \RuntimeException(sprintf('%s declares no value at all', $declared));
            }

            return $cases[0];
        }
        if (!class_exists($declared)) {
            throw new \RuntimeException(sprintf('the generator wrote a `%s` nothing here can build', $declared));
        }

        return self::built($declared, $optionals, $depth + 1);
    }

    /** One value of a schema, every member of it read off the constructor the generator wrote. */
    private static function built(string $declared, bool $optionals, int $depth): object
    {
        $reflection = new \ReflectionClass($declared);
        $constructor = $reflection->getConstructor();
        if ($constructor === null) {
            throw new \RuntimeException(sprintf('%s declares no members at all', $declared));
        }

        $types = self::declaredTypes($constructor);
        $held = [];
        foreach ($constructor->getParameters() as $parameter) {
            $name = $parameter->getName();
            if (!isset($types[$name])) {
                throw new \RuntimeException(sprintf('%s says nothing about `%s`', $declared, $name));
            }
            $held[$name] = $parameter->isDefaultValueAvailable() && !$optionals
                ? null
                : self::valueOf(self::peeled($types[$name]), $optionals, self::MODEL_TEXT, $depth);
        }

        return $reflection->newInstanceArgs($held);
    }

    /**
     * What a value is when it is there.
     *
     * Whether it may be absent is not read here: the language folds absence into `mixed` and spells
     * it `|null` everywhere else, so what says a member may be left out is the default the generator
     * wrote for it rather than the type it wrote beside it.
     */
    public static function peeled(string $type): string
    {
        return str_ends_with($type, '|null') ? substr($type, 0, -strlen('|null')) : $type;
    }

    /**
     * The name a type is declared under, spelled the way the language reaches it.
     *
     * A docblock names what the generator wrote by its short name, since the file it wrote it in
     * declares into the namespace it lands in; reflection answers the whole of it. Both arrive
     * here, and both have to name the same class.
     */
    private static function resolved(string $type): string
    {
        return str_contains($type, '\\') ? ltrim($type, '\\') : 'Hook0\\Generated\\' . $type;
    }
}

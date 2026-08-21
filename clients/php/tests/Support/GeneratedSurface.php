<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

/**
 * Everything the generator wrote, found by looking at what it wrote.
 *
 * Nothing lists the types anywhere: a schema the document adds joins the suites that read this the
 * moment the generated files carry it, and one the document drops takes its coverage with it rather
 * than leaving a case that runs against nothing.
 */
final class GeneratedSurface
{
    /** Where the generator lands, from the directory this file sits in. */
    private const ROOT = __DIR__ . '/../../src/Generated';

    /** The namespace those files declare into. */
    private const NAMESPACE = 'Hook0\\Generated\\';

    /** Most files read back out of the generated tree, far above what the API declares. */
    private const MAX_FILES = 4096;

    /**
     * Every class and enumeration the generator declared, by name.
     *
     * @return list<class-string>
     */
    public static function declared(): array
    {
        $files = glob(self::ROOT . '/*.php');
        if ($files === false || $files === []) {
            throw new \RuntimeException('the generated tree carries nothing');
        }
        if (count($files) > self::MAX_FILES) {
            throw new \RuntimeException(sprintf(
                'the generated tree carries more than the %d files read back',
                self::MAX_FILES
            ));
        }

        sort($files);
        $declared = [];
        foreach ($files as $file) {
            /** @var class-string $name */
            $name = self::NAMESPACE . basename($file, '.php');
            $declared[] = $name;
        }

        return $declared;
    }

    /**
     * Every value the API declares, which is every generated class that reads one out of a document.
     *
     * @return list<class-string>
     */
    public static function models(): array
    {
        return array_values(array_filter(
            self::declared(),
            static fn (string $name): bool => class_exists($name) && method_exists($name, 'fromJson')
        ));
    }

    /**
     * Every group of operations the generator wrote, which is every class it hands a transport to.
     *
     * @return list<class-string>
     */
    public static function groups(): array
    {
        $found = array_values(array_filter(self::declared(), static function (string $name): bool {
            if (!class_exists($name)) {
                return false;
            }
            $constructor = (new \ReflectionClass($name))->getConstructor();
            if ($constructor === null) {
                return false;
            }
            $parameters = $constructor->getParameters();
            $carried = count($parameters) === 1 ? $parameters[0]->getType() : null;

            return $carried instanceof \ReflectionNamedType && $carried->getName() === \Hook0\Transport::class;
        }));
        if ($found === []) {
            throw new \RuntimeException('the generator wrote no group of operations at all');
        }

        return $found;
    }

    /**
     * Every operation one group carries, under the name it is called by.
     *
     * @param class-string $group
     * @return list<\ReflectionMethod>
     */
    public static function operationsOf(string $group): array
    {
        $found = array_values(array_filter(
            (new \ReflectionClass($group))->getMethods(\ReflectionMethod::IS_PUBLIC),
            static fn (\ReflectionMethod $method): bool => !$method->isConstructor()
                && $method->getDeclaringClass()->getName() === $group
        ));
        usort(
            $found,
            static fn (\ReflectionMethod $one, \ReflectionMethod $other): int
                => strcmp($one->getName(), $other->getName())
        );

        return $found;
    }

    /**
     * Every closed list of strings the API declares.
     *
     * @return list<class-string<\BackedEnum>>
     */
    public static function enumerations(): array
    {
        /** @var list<class-string<\BackedEnum>> $found */
        $found = array_values(array_filter(
            self::declared(),
            static fn (string $name): bool => enum_exists($name)
        ));

        return $found;
    }
}

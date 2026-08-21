<?php

declare(strict_types=1);

namespace Hook0\Tests;

use PHPUnit\Framework\TestCase;

/**
 * What installing this package is allowed to drag in, which is nothing.
 *
 * The package reaches the network, verifies signatures and decodes what the API answers with the
 * language and the extensions every distribution of it ships. That sentence is worth exactly as much
 * as the guard behind it, so it is a case rather than a line in a pipeline: a package appearing under
 * `require` fails here, wherever the suite runs.
 */
final class ComposerTest extends TestCase
{
    /** Largest manifest read back, which is far above what one of this shape ever is. */
    private const MAX_MANIFEST_BYTES = 64 * 1024;

    public function testThePackageDeclaresNoRuntimeDependency(): void
    {
        $declared = array_keys($this->manifest()['require']);
        $packages = array_values(array_filter(
            $declared,
            static fn (string $name): bool => $name !== 'php' && !str_starts_with($name, 'ext-')
        ));

        self::assertSame(
            [],
            $packages,
            'the package has grown a runtime dependency; it is meant to reach for the language alone'
        );
    }

    public function testEveryExtensionThePackageRequiresIsOneItRunsOn(): void
    {
        // A requirement nothing loads is a requirement that stops an installation for no reason, and
        // one that is missing lets an installation succeed and the first request fail.
        foreach (array_keys($this->manifest()['require']) as $name) {
            if (!str_starts_with($name, 'ext-')) {
                continue;
            }
            self::assertTrue(extension_loaded(substr($name, 4)), $name);
        }
    }

    public function testThePackageRunsOnTheVersionItAsksFor(): void
    {
        $wanted = $this->manifest()['require']['php'];

        self::assertMatchesRegularExpression('/^>=8\.\d+$/', $wanted);
        self::assertTrue(
            version_compare(PHP_VERSION, substr($wanted, 2), '>='),
            sprintf('the suite runs on %s, below the %s the package asks for', PHP_VERSION, $wanted)
        );
    }

    public function testBothHalvesOfWhatThePackageIsAreReachable(): void
    {
        // The hand-written half and the generated one are found through the same autoloading rule, so
        // a caller reaches either without knowing which is which.
        self::assertSame(['Hook0\\' => 'src/'], $this->manifest()['autoload']['psr-4']);

        self::assertTrue(class_exists(\Hook0\Client::class));
        self::assertTrue(class_exists(\Hook0\Signature::class));
        self::assertTrue(class_exists(\Hook0\Generated\ProblemError::class));
        self::assertTrue(enum_exists(\Hook0\Generated\ProblemId::class));
    }

    /**
     * @return array<string, mixed>
     */
    private function manifest(): array
    {
        $path = __DIR__ . '/../composer.json';
        $size = filesize($path);

        self::assertNotFalse($size);
        self::assertLessThanOrEqual(self::MAX_MANIFEST_BYTES, $size);

        $read = json_decode((string) file_get_contents($path), true, 32, JSON_THROW_ON_ERROR);
        self::assertIsArray($read);

        return $read;
    }
}

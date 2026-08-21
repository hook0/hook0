<?php

/**
 * How the suite finds both halves of the package.
 *
 * The rule is the one `composer.json` declares, written out here so that the suite runs wherever the
 * package does — with a test runner and nothing else installed beside it. An installation that went
 * through Composer has its own autoloader registered already; this one answers for the classes it
 * knows and stays out of the way for everything else, so the two never disagree.
 */

declare(strict_types=1);

spl_autoload_register(static function (string $declared): void {
    $roots = [
        'Hook0\\Tests\\' => __DIR__ . '/',
        'Hook0\\' => __DIR__ . '/../src/',
    ];

    foreach ($roots as $prefix => $root) {
        if (!str_starts_with($declared, $prefix)) {
            continue;
        }

        $path = $root . str_replace('\\', '/', substr($declared, strlen($prefix))) . '.php';
        if (is_file($path)) {
            require $path;
        }

        return;
    }
});

<?php

/**
 * Starts the fake API in a process of its own.
 *
 * Nothing is declared here: this file only runs, so that the server itself stays a class the
 * autoloader finds like any other and this stays the one line that starts it.
 */

declare(strict_types=1);

require __DIR__ . '/../bootstrap.php';

exit(Hook0\Tests\Support\FakeApiServer::run(array_slice($argv, 1)));

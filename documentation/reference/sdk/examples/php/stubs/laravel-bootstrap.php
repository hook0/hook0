<?php

// Real, but inert, stand-ins for the handful of Laravel symbols the Laravel example on
// documentation/reference/sdk/php.md reaches that cannot be resolved from an installable package:
// `illuminate/foundation`, where Laravel's container-bound helpers and its base CSRF middleware
// live, stopped being published as its own Composer package after v1.1.2 and today ships bundled
// inside `laravel/framework` only. Everything else that example touches -- `Illuminate\Http\Request`,
// the `Route` facade -- is the real class from the real package required in composer.json.
//
// Loaded as a PHPStan bootstrap file, so these are declared, never executed: analysis only resolves
// them, it never calls them.

namespace {
    if (!function_exists('config')) {
        function config(array|string|null $key = null, mixed $default = null): mixed
        {
            return $default;
        }
    }

    if (!function_exists('response')) {
        function response(
            string $content = '',
            int $status = 200,
            array $headers = []
        ): \Illuminate\Contracts\Routing\ResponseFactory {
            throw new \RuntimeException('bootstrap stand-in, never called');
        }
    }

    if (!function_exists('dispatch')) {
        function dispatch(mixed $job): mixed
        {
            return null;
        }
    }
}

namespace Illuminate\Foundation\Http\Middleware {
    if (!class_exists(VerifyCsrfToken::class, false)) {
        class VerifyCsrfToken
        {
        }
    }
}

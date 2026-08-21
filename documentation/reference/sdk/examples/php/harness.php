// The rest of the file, for every PHP example of the SDK reference.
//
// A snippet on a page is written for a reader: some are complete, copy-paste-able scripts and
// already carry their own `<?php` tag and `use` imports; the rest are body fragments that assume a
// `<?php` context the reader already has open, the way the surrounding prose describes them. Each
// region below is the file that snippet would live in, with a hole where it goes. The page points
// at one by name on the fence, so what a snippet is standing on is one word away from the snippet
// itself.
//
// This file is never parsed as it stands -- the marker comments below are read as plain text, not
// as PHP, which is what lets a region reopen `<?php` where the snippet it holds already opens one
// of its own. Each region becomes a file of its own under `generated/`, resolved by PHPStan against
// the real client through the path repository in composer.json.

// HARNESS send
<?php

$applicationId = '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21';
$token = 'a-service-token';

?>
EXAMPLE

// END HARNESS

// HARNESS event
<?php

use Hook0\Event;

// The value the page shows, constructed and discarded, so every field of it is checked against the
// client.
EXAMPLE

// END HARNESS

// HARNESS bounds
<?php

$applicationId = '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21';
$token = 'a-service-token';

EXAMPLE

// END HARNESS

// HARNESS verify
<?php

/**
 * A stand-in for whatever request object a caller's own framework hands it; the page shows the
 * shape any of them can be read through, not a class this client declares.
 */
final class ExampleRequest
{
    public function header(string $name): string
    {
        return '';
    }

    public function body(): string
    {
        return '';
    }

    /** @return array<string, string> */
    public function headers(): array
    {
        return [];
    }
}

$request = new ExampleRequest();
$subscriptionSecret = 'a-subscription-secret';

EXAMPLE

// END HARNESS

// HARNESS laravel
<?php

use Illuminate\Foundation\Http\Middleware\VerifyCsrfToken;
use Illuminate\Support\Facades\Route;

/** Whatever job class the application already defines for handling a verified delivery. */
final class HandleWebhook
{
    public function __construct(public readonly array $payload)
    {
    }
}

EXAMPLE

// END HARNESS

// HARNESS upsert
<?php

use Hook0\Client;

$client = new Client(
    'https://app.hook0.com/api/v1',
    '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21',
    'a-service-token',
);

EXAMPLE

// END HARNESS

// HARNESS api
<?php

$applicationId = '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21';
$token = 'a-service-token';

EXAMPLE

// END HARNESS

// HARNESS errors
<?php

use Hook0\Client;
use Hook0\Event;
use Psr\Log\NullLogger;

$client = new Client(
    'https://app.hook0.com/api/v1',
    '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21',
    'a-service-token',
);
$event = new Event(
    eventType: 'billing.invoice.paid',
    payload: '{"invoice": "in_123"}',
    payloadContentType: 'application/json',
);
$logger = new NullLogger();

EXAMPLE

// END HARNESS

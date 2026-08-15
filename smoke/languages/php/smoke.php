<?php

declare(strict_types=1);

// The PHP client against a Hook0 that is really running.
//
// Three things the loopback suite cannot ask: whether an application secret the API minted is
// accepted, whether a second send under an identifier already ingested is reported as the conflict
// it is, and whether a signature the output worker computed verifies. Everything else about this
// client is settled by `clients/php/tests`.

require __DIR__ . '/../../../clients/php/vendor/autoload.php';

use Hook0\Client;
use Hook0\ClientError;
use Hook0\Event;
use Hook0\Signature;

/** The conflict the API answers a duplicated ingestion with. */
const ALREADY_INGESTED = 'EventAlreadyIngested';

/** A setting the harness passes, or a refusal naming it. */
function setting(string $name): string
{
    $value = getenv($name);
    if ($value === false || $value === '') {
        fwrite(STDERR, "$name is not set\n");
        exit(1);
    }
    return $value;
}

/** The event both sends carry, under the identifier the caller names. */
function event(string $eventType, ?string $eventId): Event
{
    return new Event(
        eventType: $eventType,
        payload: '{"from":"the php smoke"}',
        payloadContentType: 'application/json',
        labels: ['language' => 'php'],
        eventId: $eventId,
    );
}

/** Verifies what the output worker really delivered, with this client's own verification. */
function verify(string $delivery): void
{
    $read = static fn(string $part): string => (string) file_get_contents($delivery . '/' . $part);

    $headers = [];
    foreach (explode("\n", $read('headers')) as $line) {
        $at = strpos($line, ': ');
        if ($at !== false) {
            $headers[substr($line, 0, $at)] = substr($line, $at + 2);
        }
    }

    Signature::verify(
        trim($read('signature')),
        $read('body'),
        $headers,
        trim($read('secret')),
        (float) trim($read('tolerance')),
    );
}

$client = new Client(
    setting('HOOK0_API_URL'),
    setting('HOOK0_APPLICATION_ID'),
    setting('HOOK0_TOKEN'),
);
$eventType = setting('HOOK0_EVENT_TYPE');

$sent = $client->sendEvent(event($eventType, null));
echo "ingested $sent\n";

try {
    $client->sendEvent(event($eventType, $sent));
    fwrite(STDERR, "sending the same event twice was accepted twice\n");
    exit(1);
} catch (ClientError $refused) {
    $said = $refused->getMessage();
    if (!str_contains($said, ALREADY_INGESTED)) {
        fwrite(STDERR, 'the second send failed without naming ' . ALREADY_INGESTED . ": $said\n");
        exit(1);
    }
    echo 'the second send reported ' . ALREADY_INGESTED . "\n";
}

verify(setting('HOOK0_DELIVERY'));
echo "the signature the instance produced verifies\n";

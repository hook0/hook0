<?php

/**
 * What the dashboard shows under "Send an event", for PHP.
 *
 * This file exists so that the snippet is held against the real client. A renamed argument, a moved
 * constructor or a dropped field turns `clients.php.check` red on the day it happens, which is the
 * whole reason the snippet lives here rather than in the dashboard: one written by hand over there
 * is backed by nothing and drifts in silence.
 *
 * Two pairs of markers say how it is read. `hook0:snippet` delimits what a reader is shown, so that
 * anything this file needs only in order to be checked stays out of it. `hook0:label` delimits the
 * one rendering of a label, which the dashboard repeats once per label the form carries and joins
 * with the separator its manifest declares — the region carries no trailing separator of its own,
 * and sits inside its container, so no label at all leaves a valid empty one.
 *
 * The region opens after the `<?php` rather than before it, which is the one place a PHP file
 * cannot carry a comment: anything ahead of the opening tag is output rather than code. A reader
 * pasting the snippet into a file of their own writes that line themselves, as they would for any
 * PHP source.
 *
 * `declare(strict_types=1)` sits outside for a different reason: it has to be the very first
 * statement of a file, so a snippet carrying it cannot go into a file that already has one —
 * that is a fatal error rather than a lint, and the reader would meet it on their own code.
 *
 * The `__HOOK0_*__` words are string literals, which is what lets a file full of them be read as
 * PHP. They never resolve to anything: this example is checked, never run.
 */

declare(strict_types=1);

// hook0:snippet:begin
use Hook0\Client;
use Hook0\Event;

$client = new Client(
    "__HOOK0_API_URL__",
    "__HOOK0_APPLICATION_ID__",
    "__HOOK0_TOKEN__",
);

$eventId = $client->sendEvent(new Event(
    eventType: "__HOOK0_EVENT_TYPE__",
    payload: "__HOOK0_PAYLOAD__",
    payloadContentType: "application/json",
    labels: [
        // hook0:label:begin
        "__HOOK0_LABEL_KEY__" => "__HOOK0_LABEL_VALUE__", // hook0:label:end
    ],
));

echo "ingested as {$eventId}\n";
// hook0:snippet:end

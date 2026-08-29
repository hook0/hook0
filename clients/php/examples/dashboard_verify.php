<?php

/**
 * What the dashboard shows under "Verify a webhook", for PHP.
 *
 * Sending is only half of what a reader has come to do, and it is the easier half. This is the one
 * the SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside
 * the send rather than leaving it to be found later.
 *
 * The secret is read from the environment on purpose. The dashboard cannot know which subscription
 * a reader means — outside the onboarding it loads none, and an application may have several — so
 * it points at the subscription instead of guessing one, and no second secret is put on screen.
 *
 * `declare(strict_types=1)` sits outside for a different reason: it has to be the very first
 * statement of a file, so a snippet carrying it cannot go into a file that already has one —
 * that is a fatal error rather than a lint, and the reader would meet it on their own code.
 *
 * Read the markers as in `dashboard_send.php`: `hook0:snippet` is what is displayed, everything
 * outside it is what holds this file against the client. Nothing here declares a symbol and runs
 * something too, which is what `PSR1.Files.SideEffects` refuses — so where the send example is
 * statements alone, this one is declarations alone and calls neither.
 */

declare(strict_types=1);

// hook0:snippet:begin
use Hook0\ClientError;
use Hook0\Signature;

/**
 * Verify against the *raw* body: one that has been parsed and serialised again no longer hashes to
 * what was signed. The tolerance is bilateral, so a delivery dated too far ahead is refused exactly
 * like one dated too far behind.
 *
 * @param array<string, string> $headers the headers the delivery carried
 */
function accept(string $signature, string $body, array $headers): bool
{
    // The secret of the subscription being verified, which the dashboard links to rather than
    // prints: it cannot know which subscription a reader means, and an application may have several.
    // A variable nobody exported and one exported empty are the same defect and are refused
    // together: verification hashes the delivery against whatever key it is handed, so an empty one
    // refuses every genuine delivery as forged and nothing says the variable was never read.
    //
    // The raise names the global class outright, because this is pasted into a file that has a
    // namespace of its own: unqualified, it would resolve there instead and the raise would become
    // a class that does not exist.
    $secret = getenv('HOOK0_SUBSCRIPTION_SECRET');
    if ($secret === false || $secret === '') {
        throw new \RuntimeException('HOOK0_SUBSCRIPTION_SECRET is not set');
    }

    try {
        Signature::verify($signature, $body, $headers, $secret, 300.0);
    } catch (ClientError) {
        return false;
    }

    return true;
}
// hook0:snippet:end

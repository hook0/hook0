/**
 * What the dashboard shows under "Verify a webhook", for TypeScript.
 *
 * Sending is only half of what a reader has come to do, and it is the easier half. This is the one
 * the SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside
 * the send rather than leaving it to be found later.
 *
 * The secret is read from the environment on purpose. The dashboard cannot know which subscription a
 * reader means — outside the onboarding it loads none, and an application may have several — so it
 * points at the subscription instead of guessing one, and no second secret is put on screen.
 *
 * Read the markers as in `dashboard_send.ts`: `hook0:snippet` is what is displayed, everything
 * outside it is what makes the file type-check.
 */

// hook0:snippet:begin
import { verifyWebhookSignature } from 'hook0-client';

// Verify against the *raw* body: one that has been parsed and serialised again no longer hashes to
// what was signed. Every refusal is thrown rather than returned, so there is no falsy answer to
// branch on and a handler without a `try` treats every forged delivery as a crash. The tolerance is
// a number of seconds and the window is bilateral, so a delivery dated too far ahead is refused
// exactly like one dated too far behind.
export function accepted(signature: string, rawBody: Buffer, headers: Headers): boolean {
  // The secret of the subscription being verified, which the dashboard links to rather than
  // prints. Checked rather than asserted, and above the `try` rather than inside it. An assertion
  // tells the compiler the variable is set without making it so, and a variable nobody exported and
  // one exported empty are the same defect: either hashes every genuine delivery to the wrong code,
  // and raising from inside the `try` would come back as a refused delivery like any other.
  const secret = process.env.HOOK0_SUBSCRIPTION_SECRET;
  if (secret === undefined || secret.length === 0) {
    throw new Error('HOOK0_SUBSCRIPTION_SECRET is not set');
  }

  try {
    verifyWebhookSignature(signature, rawBody, headers, secret, 300);
    return true;
  } catch {
    return false;
  }
}
// hook0:snippet:end

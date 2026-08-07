// Relative on purpose: this module is unit-tested, and jest resolves paths
// without the bundler's `@/` alias.
import { NO_COOLDOWN, type CooldownStart } from './cooldown';

/**
 * What the check-email page was handed by the page that redirected to it.
 *
 * A tagged union rather than a string that may be empty: "nobody handed an
 * address over" (a bookmarked `/check-email`, a deep link) is a distinct case
 * with its own rendering, and no address value is overloaded to mean absence.
 */
export type ResendTarget =
  | {
      readonly kind: 'address';
      /** The address a resend would go to. */
      readonly email: string;
      /**
       * When the redirecting page last caused a verification email to be sent to
       * that address, as far as it knows. Signing up sends one right then;
       * being bounced off the login form sends nothing.
       */
      readonly lastVerificationSend: CooldownStart;
    }
  | { readonly kind: 'none' };

/** The single "no address was handed over" value. */
export const NO_RESEND_TARGET: ResendTarget = { kind: 'none' };

/**
 * The History API state that hands `email` to the check-email page, declaring
 * when the caller just sent a verification email to it (`NO_COOLDOWN` when it
 * sent none).
 *
 * History API state, never the URL: vue-matomo auto-tracks the SPA URL — query
 * string included — as the Matomo page/referrer URL, so an `?email=` query would
 * ship the user's address to analytics. State survives a reload, so the resend
 * flow stays refresh-safe.
 *
 * Declaring the send is what keeps the resend button honest. The endpoint
 * answers 204 whether or not it actually sent (anti-enumeration), so a button
 * offered while the server is still in its cooldown reports a send that never
 * happened.
 */
export function checkEmailHandoverState(
  email: string,
  lastVerificationSend: CooldownStart
): Record<string, string | number> {
  if (lastVerificationSend.kind === 'started') {
    return { email, verificationEmailSentAtMs: lastVerificationSend.atMs };
  }
  return { email };
}

/**
 * Read back what a redirect handed over, from `window.history.state` or anything
 * else claiming to be it.
 *
 * The state is untrusted input — a restored session, a hand-edited entry, a
 * router that added its own bookkeeping keys — so anything that is not an intact
 * address reads as "none", and a send stamp that is not a finite number reads as
 * "no send declared". Both degrade to the previous behaviour (button hidden, or
 * button live and the server the sole arbiter) rather than to a broken page.
 */
export function readCheckEmailHandover(state: unknown): ResendTarget {
  if (state === null || typeof state !== 'object') {
    return NO_RESEND_TARGET;
  }

  const record = state as Record<string, unknown>;
  const email = record.email;
  if (typeof email !== 'string' || email.length === 0) {
    return NO_RESEND_TARGET;
  }

  const sentAtMs = record.verificationEmailSentAtMs;
  if (typeof sentAtMs !== 'number' || !Number.isFinite(sentAtMs)) {
    return { kind: 'address', email, lastVerificationSend: NO_COOLDOWN };
  }

  return {
    kind: 'address',
    email,
    lastVerificationSend: { kind: 'started', atMs: sentAtMs },
  };
}

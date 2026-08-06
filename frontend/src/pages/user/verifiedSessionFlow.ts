import type { components } from '../../types';

type LoginResponse = components['schemas']['LoginResponse'];

export interface CompleteVerifiedSessionDeps {
  /** The session returned by a successful e-mail verification. */
  session: LoginResponse;
  /** Establish the authenticated session (persist tokens, schedule refresh…). */
  setSession: (session: LoginResponse) => Promise<void>;
  /** Post-auth navigation probe (list orgs/apps, then route accordingly). */
  navigateAfterAuth: () => Promise<unknown>;
  /** Fallback navigation to Home once the session already exists. */
  goHome: () => Promise<unknown>;
}

/**
 * Finalize a successful e-mail verification.
 *
 * Establishes the authenticated session, then runs the post-auth navigation
 * probe. Once `setSession` has succeeded the user IS authenticated, so a
 * failure of the navigation probe must not bounce them back to the login form:
 * we log it and fall back to Home, and the returned promise still resolves.
 *
 * A failure from `setSession` itself (session never established) propagates so
 * the caller can surface it as a pre-session verification error.
 */
export function completeVerifiedSession(deps: CompleteVerifiedSessionDeps): Promise<unknown> {
  const { session, setSession, navigateAfterAuth, goHome } = deps;
  return setSession(session).then(() =>
    navigateAfterAuth().catch((navErr) => {
      console.error(navErr);
      return goHome();
    })
  );
}

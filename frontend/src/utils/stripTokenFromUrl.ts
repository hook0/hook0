import type { Router } from 'vue-router';

/**
 * Drop the `token` query parameter from the current URL, in place.
 *
 * Pages reached from a link in an email carry a single-use credential in their
 * query string. Anything that observes URLs observes that credential: the
 * analytics plugin tracks `to.fullPath` on every navigation and reports the
 * previous `fullPath` as the referrer of the next one, and the browser keeps it
 * in history. Marking the route `analyticsIgnore` stops the page view but not
 * the referrer of whatever the user visits next, so the token has to leave the
 * address bar itself.
 *
 * `router.replace` rather than `history.replaceState`, because the router keeps
 * its own copy of the current location and that copy is what the referrer is
 * read from. Replacing the query on the same route does not remount the page,
 * so callers can read the token first and strip it immediately after.
 */
export function stripTokenFromUrl(router: Router): void {
  const current = router.currentRoute.value;
  if (!('token' in current.query)) {
    return;
  }

  const { token: _token, ...rest } = current.query;
  void router.replace({ path: current.path, query: rest, hash: current.hash });
}

import { createRouter, createMemoryHistory } from 'vue-router';

/**
 * Privacy guard for the check-email page.
 *
 * vue-matomo (configured with the `router` option in src/plugins/matomo.ts)
 * auto-tracks every SPA page view. The URL it records is
 * `window.location.origin + getResolvedHref(router, to.fullPath)`, and
 * getResolvedHref is just `router.resolve(path).href` (see
 * node_modules/vue-matomo/src/index.js + utils.js). The previous page it records
 * as the referrer is derived the same way from `from.fullPath`. So ANYTHING in a
 * route's URL — path, query string OR hash — is shipped to Matomo.
 *
 * The signup address must never reach analytics. It is therefore carried to
 * /check-email through History API state (window.history.state), which is NOT
 * part of fullPath. These tests prove the URL vue-matomo would record for the
 * check-email navigation contains no email, in both the page-URL and referrer
 * positions, and pin the pre-fix query-param behaviour as the regression to
 * avoid.
 */
describe('check-email navigation keeps the email out of the tracked URL', () => {
  const CHECK_EMAIL = 'CheckEmail';
  const email = 'secret-user@example.com';
  // The '@' is percent-encoded in a query string, so assert on the local part,
  // which survives any encoding.
  const secretLocalPart = 'secret-user';

  function makeRouter() {
    return createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/check-email', name: CHECK_EMAIL, component: { render: () => null } },
        { path: '/login', name: 'Login', component: { render: () => null } },
      ],
    });
  }

  // Exactly what vue-matomo records for a page/referrer position: the custom URL
  // is origin + getResolvedHref(router, fullPath) = origin + resolve(fullPath).href.
  function trackedUrlFor(router: ReturnType<typeof makeRouter>, fullPath: string): string {
    return 'https://app.hook0.com' + router.resolve(fullPath).href;
  }

  it('carries the email via History API state, so fullPath stays clean', async () => {
    const router = makeRouter();
    await router.push({ name: CHECK_EMAIL, state: { email } });

    const fullPath = router.currentRoute.value.fullPath;
    expect(fullPath).toBe('/check-email');
    expect(fullPath).not.toContain(secretLocalPart);

    // The page URL vue-matomo would record.
    expect(trackedUrlFor(router, fullPath)).toBe('https://app.hook0.com/check-email');
    expect(trackedUrlFor(router, fullPath)).not.toContain(secretLocalPart);
  });

  it('does not leak the email as the referrer URL when leaving check-email', async () => {
    const router = makeRouter();
    await router.push({ name: CHECK_EMAIL, state: { email } });
    const from = router.currentRoute.value;
    await router.push({ name: 'Login' });

    // vue-matomo records the previous page as the referrer via `from.fullPath`.
    expect(trackedUrlFor(router, from.fullPath)).not.toContain(secretLocalPart);
  });

  it('would have leaked the email through the URL had it stayed a query param', () => {
    // Pre-fix behaviour, kept as a red line: a query param lands in fullPath and
    // therefore in the Matomo custom URL.
    const router = makeRouter();
    const leaky = router.resolve({ name: CHECK_EMAIL, query: { email } });

    expect(leaky.fullPath).toContain(secretLocalPart);
    expect(trackedUrlFor(router, leaky.fullPath)).toContain(secretLocalPart);
  });
});

// Docusaurus client module: restore Scalar API-reference deep links on hard load.
//
// The API reference at /api is a Scalar viewer (@scalar/api-reference, pulled
// from the jsDelivr CDN by @scalar/docusaurus) that fetches its OpenAPI
// specification, hook0-api.json, at runtime. On an in-app click Scalar already
// holds the spec in memory, so a permalink such as
//   /api#tag/subscriptions-management/GET/api/v1/request_attempts
// selects the operation and scrolls to it. On a hard load or refresh of that
// same URL, Scalar resolves the initial hash against a navigation tree the spec
// has not populated yet, falls back to the reference overview, and never re-runs
// that initial navigation once the spec finishes loading. The endpoint the link
// points at is therefore never shown. See HOO-123 / GitLab issue #123.
//
// Scalar drives its own navigation from the current URL on `popstate`: its
// handler reads window.location.href and selects the matching operation. Once
// the spec has rendered we re-dispatch a popstate, which makes Scalar navigate
// to the hash with its own logic (select, expand, scroll) instead of us reaching
// into its internal DOM. The nudge is a no-op when the first navigation already
// landed on the right operation, since it re-selects the same one.

// The Scalar route is <baseUrl>api. baseUrl is "/" in production; match "/api"
// with or without a trailing slash and nothing else after it.
function isApiReferenceRoute() {
  return /(^|\/)api\/?$/.test(window.location.pathname);
}

// Scalar gives every operation a DOM id equal to its anchor slug ("tag/..."),
// which is what makes the in-app anchors resolvable. Their presence is the
// signal that the spec loaded and the navigation tree is populated.
function specHasRendered() {
  return document.querySelector('[id^="tag/"]') !== null;
}

function restoreDeepLink() {
  if (!isApiReferenceRoute() || !window.location.hash) {
    return;
  }

  var deadline = Date.now() + 8000; // give a slow spec fetch time to finish
  var nudged = false;

  function tick() {
    if (!window.location.hash) {
      return; // the reader navigated away from the anchor
    }

    if (specHasRendered()) {
      if (!nudged) {
        nudged = true;
        // Let Scalar re-read the location and navigate to the hash itself.
        window.dispatchEvent(new PopStateEvent('popstate'));
      }

      var target = document.getElementById(
        decodeURIComponent(window.location.hash.slice(1))
      );
      if (target) {
        target.scrollIntoView({ block: 'start' });
        return;
      }
    }

    if (Date.now() < deadline) {
      window.requestAnimationFrame(tick);
    }
  }

  window.requestAnimationFrame(tick);
}

// Fires on the initial render and on every client-side route change, matching
// how the Mermaid theme-switcher module hooks the same lifecycle.
export function onRouteDidUpdate() {
  if (typeof window === 'undefined') {
    return;
  }
  restoreDeepLink();
}

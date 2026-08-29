import type { Page } from "@playwright/test";

/**
 * Record what the app pushes to `window._paq`, in a log at
 * `window.__trackedEvents` that nothing else can reach.
 *
 * Creating `window._paq` is what makes the tracking call observable, since
 * `trackEvent` only pushes when it exists — the call itself is the real one.
 * Getting it back out took three tries, because the Matomo snippet (loaded
 * whenever `/instance` advertises a Matomo config) is an active participant:
 * it replaces `window._paq` with its own tracker object, and when it cannot,
 * it consumes the queue in place and leaves holes in it.
 *
 * So the queue is pinned (the setter drops the replacement, so the identity
 * never changes mid-test) and every push is copied into a separate log that
 * the snippet has no reference to and cannot rewrite. The assertion reads the
 * log, never the queue.
 */
export async function captureTrackingQueue(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const log: unknown[] = [];
    (window as unknown as { __trackedEvents: unknown[] }).__trackedEvents = log;

    const queue: unknown[] = [];
    const push = queue.push.bind(queue);
    queue.push = (...entries: unknown[]) => {
      log.push(...entries);
      return push(...entries);
    };

    Object.defineProperty(window, "_paq", {
      configurable: true,
      get: () => queue,
      set: () => {
        // Keep ours: see above.
      },
    });
  });
}

/**
 * Every `trackEvent` pushed since the page was created, as the argument arrays
 * Matomo receives: `["trackEvent", category, action, name]`.
 *
 * Reading the log rather than the queue is the whole point of the pinning
 * above, so this never touches `window._paq`.
 */
export async function trackedEvents(page: Page): Promise<unknown[][]> {
  return page.evaluate(() => {
    const log = (window as unknown as { __trackedEvents?: unknown[] }).__trackedEvents ?? [];
    return log
      .filter((entry): entry is unknown[] => Array.isArray(entry))
      .filter((entry) => entry[0] === "trackEvent");
  });
}

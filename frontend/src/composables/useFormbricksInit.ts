import formbricks from '@formbricks/js';
import { getInstanceConfig } from '@/utils/instance-config';

/**
 * Initialize Formbricks survey SDK if the instance is configured for it.
 * Safe to call server-side (no-ops when `window` is undefined).
 */
export function initializeFormbricks(): Promise<void> {
  if (typeof window === 'undefined') return Promise.resolve();

  return getInstanceConfig().then((config) => {
    if (
      config &&
      config.formbricks &&
      config.formbricks.api_host &&
      config.formbricks.environment_id
    ) {
      return formbricks
        .setup({
          appUrl: config.formbricks.api_host,
          environmentId: config.formbricks.environment_id,
        })
        .catch((e) => {
          console.warn(`Formbricks initialization failed: ${e}`);
        });
    }
  });
}

/**
 * Track a route change in Formbricks (call inside router.beforeEach).
 */
export function trackFormbricksRoute(): void {
  if (!window.formbricks) return;
  formbricks.registerRouteChange().catch((e) => {
    console.warn(`Formbricks register route change failed: ${e}`);
  });
  formbricks.track('route_changed').catch((e) => {
    console.warn(`Formbricks track failed: ${e}`);
  });
}

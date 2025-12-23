/**
 * Matomo event tracking composable
 *
 * Custom Dimensions:
 * - 1: page_category (auth, dashboard, organization, application, tutorial, api-docs)
 * - 2: page_action (view, create, edit, delete, list)
 * - 3: funnel_step (signup-form, signup-submit, email-sent, email-verified)
 */

declare global {
  interface Window {
    _paq?: Array<Array<string | number | undefined>>;
  }
}

export function useTracking() {
  /**
   * Track a custom event in Matomo
   * @param category - Event category (e.g., 'Signup', 'Organization', 'Application')
   * @param action - Event action (e.g., 'FormSubmit', 'Create', 'Delete')
   * @param name - Optional event name for additional context
   * @param value - Optional numeric value
   */
  function trackEvent(category: string, action: string, name?: string, value?: number) {
    if (window._paq) {
      window._paq.push(['trackEvent', category, action, name, value]);
    }
  }

  /**
   * Set a custom dimension for the current page view
   * @param id - Dimension ID (1-3)
   * @param value - Dimension value
   */
  function setCustomDimension(id: number, value: string) {
    if (window._paq) {
      window._paq.push(['setCustomDimension', id, value]);
    }
  }

  /**
   * Track page view with custom dimensions
   * @param category - Page category for dimension 1
   * @param action - Page action for dimension 2
   * @param funnelStep - Optional funnel step for dimension 3
   */
  function trackPageWithDimensions(category: string, action: string, funnelStep?: string) {
    setCustomDimension(1, category);
    setCustomDimension(2, action);
    if (funnelStep) {
      setCustomDimension(3, funnelStep);
    }
  }

  return {
    trackEvent,
    setCustomDimension,
    trackPageWithDimensions,
  };
}

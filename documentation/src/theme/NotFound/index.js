/**
 * NotFound component wrapper for 404 tracking (Recommendation 7)
 *
 * This component wraps the default Docusaurus NotFound component
 * to track 404 errors in Matomo.
 */
import React, { useEffect } from 'react';
import NotFound from '@theme-original/NotFound';

export default function NotFoundWrapper(props) {
  useEffect(() => {
    // Track 404 error in Matomo
    if (typeof window !== 'undefined' && window._paq) {
      const referrer = document.referrer || 'direct';
      const pathname = window.location.pathname;

      // Set document title for 404 page
      window._paq.push(['setDocumentTitle', '404 - Page Not Found']);

      // Track as event with referrer information
      window._paq.push([
        'trackEvent',
        'Error',
        '404',
        referrer + ' -> ' + pathname,
        1,
      ]);

      // Also track as page view so it appears in page reports
      window._paq.push(['trackPageView']);
    }
  }, []);

  return <NotFound {...props} />;
}

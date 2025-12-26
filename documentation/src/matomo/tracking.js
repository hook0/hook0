/**
 * Matomo Advanced Tracking for Hook0 Documentation
 *
 * Implements:
 * - Custom Dimensions by Diataxis content type (Recommendation 2)
 * - CTA event tracking (Recommendation 3)
 * - Scroll depth tracking (Recommendation 5)
 * - Code copy tracking (Recommendation 6)
 * - External link tracking (Recommendation 8)
 * - Content tracking (Recommendation 9)
 * - Performance tracking (Recommendation 10)
 *
 * Note: Search tracking (1) and 404 tracking (7) are handled by separate components
 */

// Diataxis content type detection (Recommendation 2)
function getContentType(pathname) {
  if (pathname.includes('/tutorials')) return 'tutorial';
  if (pathname.includes('/how-to-guides')) return 'how-to';
  if (pathname.includes('/reference') || pathname.includes('/api')) return 'reference';
  if (pathname.includes('/explanation')) return 'explanation';
  if (pathname.includes('/self-hosting')) return 'self-hosting';
  if (pathname.includes('/hook0-cloud')) return 'policies';
  if (pathname.includes('/comparisons')) return 'comparison';
  if (pathname.includes('/concepts')) return 'concepts';
  if (pathname.includes('/resources')) return 'resources';
  if (pathname === '/' || pathname === '') return 'home';
  return 'other';
}

// Safe event tracking helper
function trackEvent(category, action, name, value) {
  if (typeof window !== 'undefined' && window._paq) {
    window._paq.push(['trackEvent', category, action, name, value]);
  }
}

// Set custom dimension (Recommendation 2)
function setContentTypeDimension(pathname) {
  if (typeof window !== 'undefined' && window._paq) {
    const contentType = getContentType(pathname);
    // Custom Dimension ID 1 = Content Type (must be configured in Matomo Admin)
    window._paq.push(['setCustomDimension', 1, contentType]);
  }
}

// Scroll depth tracking (Recommendation 5)
let scrollMarks = [25, 50, 75, 100];
let firedMarks = [];
let scrollListenerAttached = false;

function resetScrollTracking() {
  firedMarks = [];
}

function handleScroll() {
  if (typeof window === 'undefined') return;

  const scrollHeight = document.documentElement.scrollHeight - window.innerHeight;
  if (scrollHeight <= 0) return;

  const scrollPercent = Math.round((window.scrollY / scrollHeight) * 100);

  scrollMarks.forEach(mark => {
    if (scrollPercent >= mark && !firedMarks.includes(mark)) {
      firedMarks.push(mark);
      trackEvent('Scroll', 'Depth', mark + '%', mark);
    }
  });
}

function initScrollTracking() {
  if (typeof window === 'undefined' || scrollListenerAttached) return;

  window.addEventListener('scroll', handleScroll, { passive: true });
  scrollListenerAttached = true;
}

// Code copy tracking (Recommendation 6)
function initCodeCopyTracking() {
  if (typeof document === 'undefined') return;

  document.addEventListener('click', function(e) {
    // Docusaurus uses button.clean-btn inside .buttonGroup__atx for copy
    const copyButton = e.target.closest('button[class*="copyButton"], button[aria-label="Copy code to clipboard"]');
    if (!copyButton) return;

    const codeBlock = copyButton.closest('[class*="codeBlock"]');
    if (!codeBlock) return;

    const codeElement = codeBlock.querySelector('code');
    const languageMatch = codeElement?.className?.match(/language-(\w+)/);
    const language = languageMatch ? languageMatch[1] : 'unknown';
    const pathname = window.location.pathname;

    trackEvent('Code', 'Copy', pathname + ' (' + language + ')', 1);
  });
}

// CTA tracking (Recommendation 3)
function initCTATracking() {
  if (typeof document === 'undefined') return;

  document.addEventListener('click', function(e) {
    const link = e.target.closest('a');
    if (!link) return;

    const href = link.getAttribute('href');
    if (!href) return;

    // Registration CTA
    if (href.includes('app.hook0.com/register')) {
      trackEvent('CTA', 'Click', 'Get Started', 1);
      return;
    }

    // Login CTA
    if (href.includes('app.hook0.com') && !href.includes('register')) {
      trackEvent('CTA', 'Click', 'Login', 1);
      return;
    }

    // Contact Support
    if (href.includes('mailto:support@hook0.com')) {
      trackEvent('CTA', 'Click', 'Contact Support', 1);
      return;
    }

    // Discord
    if (href.includes('discord.gg') || href.includes('discord.com')) {
      trackEvent('CTA', 'Click', 'Join Discord', 1);
      return;
    }
  });
}

// External link tracking (Recommendation 8)
function initExternalLinkTracking() {
  if (typeof document === 'undefined') return;

  document.addEventListener('click', function(e) {
    const link = e.target.closest('a[href^="http"]');
    if (!link) return;

    const href = link.getAttribute('href');
    if (!href) return;

    // Skip internal links
    if (link.hostname === window.location.hostname) return;

    // Skip already tracked CTAs
    if (href.includes('app.hook0.com') || href.includes('mailto:') || href.includes('discord')) return;

    // Categorize external links
    let category = 'External';
    if (href.includes('github.com') || href.includes('gitlab.com')) {
      category = 'Repository';
    } else if (href.includes('stripe.com')) {
      category = 'Payment';
    } else if (href.includes('npmjs.com') || href.includes('crates.io')) {
      category = 'Package Registry';
    } else if (href.includes('stackoverflow.com')) {
      category = 'Community';
    }

    trackEvent('Outbound', category, href, 1);
  });
}

// Content tracking (Recommendation 9)
function initContentTracking() {
  if (typeof window === 'undefined' || typeof window._paq === 'undefined') return;

  // Enable Matomo content tracking
  window._paq.push(['trackVisibleContentImpressions']);
}

// Performance tracking (Recommendation 10)
function trackPerformance() {
  if (typeof window === 'undefined' || typeof performance === 'undefined') return;

  // Wait for page load to complete
  if (document.readyState === 'complete') {
    sendPerformanceMetrics();
  } else {
    window.addEventListener('load', sendPerformanceMetrics);
  }
}

function sendPerformanceMetrics() {
  if (typeof window === 'undefined' || typeof performance === 'undefined') return;

  // Use PerformanceNavigationTiming API (modern browsers)
  const entries = performance.getEntriesByType('navigation');
  if (entries.length > 0) {
    const timing = entries[0];
    const pageLoadTime = Math.round(timing.loadEventEnd - timing.startTime);
    const dnsTime = Math.round(timing.domainLookupEnd - timing.domainLookupStart);
    const connectTime = Math.round(timing.connectEnd - timing.connectStart);
    const responseTime = Math.round(timing.responseEnd - timing.requestStart);
    const domInteractive = Math.round(timing.domInteractive - timing.startTime);

    if (pageLoadTime > 0) {
      trackEvent('Performance', 'PageLoad', window.location.pathname, pageLoadTime);
    }
    if (domInteractive > 0) {
      trackEvent('Performance', 'DOMInteractive', window.location.pathname, domInteractive);
    }
  }
}

// Track page view with custom dimension
function trackPageView(pathname) {
  if (typeof window === 'undefined' || typeof window._paq === 'undefined') return;

  setContentTypeDimension(pathname);
  window._paq.push(['setCustomUrl', pathname]);
  window._paq.push(['setDocumentTitle', document.title]);
  window._paq.push(['trackPageView']);
}

// Initialize all tracking on first load
function initTracking() {
  if (typeof window === 'undefined') return;

  // Wait for Matomo to be ready
  const checkMatomo = setInterval(() => {
    if (window._paq) {
      clearInterval(checkMatomo);

      initScrollTracking();
      initCodeCopyTracking();
      initCTATracking();
      initExternalLinkTracking();
      initContentTracking();
      trackPerformance();

      // Track initial page view with content type dimension
      setContentTypeDimension(window.location.pathname);
    }
  }, 100);

  // Give up after 5 seconds
  setTimeout(() => clearInterval(checkMatomo), 5000);
}

// Docusaurus client module exports
export function onRouteDidUpdate({ location, previousLocation }) {
  // Skip if same page
  if (location.pathname === previousLocation?.pathname) return;

  // Reset scroll tracking for new page
  resetScrollTracking();

  // Track page view with custom dimension
  // Small delay to ensure title is updated
  setTimeout(() => {
    trackPageView(location.pathname);
  }, 50);
}

// Initialize on module load
if (typeof window !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initTracking);
  } else {
    initTracking();
  }
}

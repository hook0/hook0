/**
 * Google Ads gtag.js integration for conversion tracking.
 *
 * Loaded only when VITE_GOOGLE_ADS_CONVERSION_ID is set.
 * Respects cookie consent via the same localStorage key used on the website.
 */

declare global {
  interface Window {
    dataLayer: Array<unknown>;
    gtag: (...args: unknown[]) => void;
  }
}

const CONSENT_KEY = 'hook0_cookie_consent';
const CONVERSION_ID = import.meta.env.VITE_GOOGLE_ADS_CONVERSION_ID as string | undefined;
const CONVERSION_LABEL = import.meta.env.VITE_GOOGLE_ADS_CONVERSION_LABEL as string | undefined;

let gtagLoaded = false;

function hasConsent(): boolean {
  try {
    return localStorage.getItem(CONSENT_KEY) === 'granted';
  } catch {
    return false;
  }
}

function ensureGtagFunction(): void {
  if (!window.dataLayer) {
    window.dataLayer = [];
  }
  if (!window.gtag) {
    window.gtag = function gtag() {
      // eslint-disable-next-line prefer-rest-params
      window.dataLayer.push(arguments);
    };
  }
}

function loadGtagScript(): void {
  if (gtagLoaded || !CONVERSION_ID) return;
  gtagLoaded = true;

  ensureGtagFunction();
  window.gtag('js', new Date());
  window.gtag('config', CONVERSION_ID, { send_page_view: false });

  const script = document.createElement('script');
  script.async = true;
  script.src = 'https://www.googletagmanager.com/gtag/js?id=' + CONVERSION_ID;
  document.head.appendChild(script);
}

/**
 * Initialise gtag.js if the env var is present and consent is granted.
 * Must be called once at app startup.
 */
export function setupGtag(): void {
  if (!CONVERSION_ID) return;
  ensureGtagFunction();
  if (hasConsent()) loadGtagScript();
}

/**
 * Call after the user grants cookie consent (e.g. from a consent banner
 * inside the app, if one exists).
 */
export function gtagOnConsentGranted(): void {
  loadGtagScript();
}

/**
 * Fire the primary Google Ads signup conversion.
 */
export function trackSignupConversion(): void {
  if (!gtagLoaded || !CONVERSION_ID) return;
  const sendTo = CONVERSION_LABEL
    ? CONVERSION_ID + '/' + CONVERSION_LABEL
    : CONVERSION_ID + '/signup';
  window.gtag('event', 'conversion', { send_to: sendTo });
}

/**
 * Fire a named conversion event (for secondary conversions).
 */
export function trackGtagConversion(label: string): void {
  if (!gtagLoaded || !CONVERSION_ID) return;
  window.gtag('event', 'conversion', { send_to: CONVERSION_ID + '/' + label });
}

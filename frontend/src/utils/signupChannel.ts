/**
 * Where a signup came from, as a bounded label.
 *
 * Registrations happen on app.hook0.com while web analytics watches
 * www.hook0.com, so a visitor who reaches the sign-up form without loading a
 * tracked marketing page has no recorded origin at all. This derives one from
 * what the browser already knows — the referrer and the campaign parameters of
 * the entry page — and hands it to the API, which stores it on the account.
 *
 * Only the family of the source is kept (`organic:google`, `social:linkedin`,
 * `referral:<host>`, `direct`, …): never a URL, a path, a query string or an
 * identifier. `unknown` is a value of that vocabulary rather than a stand-in
 * for a missing one — "this signup has no known origin" is itself the answer
 * to the question the column exists to ask, and it is stored and grouped like
 * any other label.
 *
 * The vocabulary is shared with `api/src/signup_channel.rs` and with the CHECK
 * constraint on `iam.user.signup_channel`: a label the API does not recognise
 * is stored as `unknown`, so a mistake here loses information without ever
 * corrupting the column.
 */

/** Stored when the origin cannot be told from the entry page. */
export const UNKNOWN_CHANNEL = 'unknown';

/** The visitor arrived with no referrer: typed address, bookmark, native app. */
const DIRECT_CHANNEL = 'direct';

/** Longest suffix the API and the database accept after the `:`. */
const MAX_SUFFIX_LENGTH = 64;

/**
 * Referrers longer than this are not parsed at all. A URL that long is not a
 * referrer any browser produced, and the parser should not be the place where
 * that gets discovered.
 */
const MAX_REFERRER_LENGTH = 2048;

/**
 * Survives navigation inside the application: the entry page carries the
 * referrer, the sign-up form is usually reached a click or two later. Session
 * scope, so it dies with the tab rather than following the visitor around.
 */
const SESSION_STORAGE_KEY = 'hook0_signup_channel';

/** Hosts whose visitors arrive from an AI assistant rather than a search page. */
const ASSISTANTS: ReadonlyArray<readonly [string, string]> = [
  ['chatgpt.com', 'chatgpt'],
  ['chat.openai.com', 'chatgpt'],
  ['openai.com', 'chatgpt'],
  ['perplexity.ai', 'perplexity'],
  ['claude.ai', 'claude'],
  ['gemini.google.com', 'gemini'],
  ['copilot.microsoft.com', 'copilot'],
  ['you.com', 'you'],
  ['phind.com', 'phind'],
];

/** Search engines, matched on their registrable domain. */
const SEARCH_ENGINES: ReadonlyArray<readonly [string, string]> = [
  ['bing.com', 'bing'],
  ['duckduckgo.com', 'duckduckgo'],
  ['ecosia.org', 'ecosia'],
  ['search.brave.com', 'brave'],
  ['search.yahoo.com', 'yahoo'],
  ['qwant.com', 'qwant'],
  ['startpage.com', 'startpage'],
  ['baidu.com', 'baidu'],
  ['yandex.com', 'yandex'],
  ['yandex.ru', 'yandex'],
];

/** Social and community sources worth telling apart from plain referrals. */
const SOCIAL: ReadonlyArray<readonly [string, string]> = [
  ['linkedin.com', 'linkedin'],
  ['lnkd.in', 'linkedin'],
  ['x.com', 'x'],
  ['twitter.com', 'x'],
  ['t.co', 'x'],
  ['reddit.com', 'reddit'],
  ['news.ycombinator.com', 'hackernews'],
  ['github.com', 'github'],
  ['youtube.com', 'youtube'],
  ['youtu.be', 'youtube'],
  ['facebook.com', 'facebook'],
  ['discord.com', 'discord'],
  ['bsky.app', 'bluesky'],
  ['mastodon.social', 'mastodon'],
  ['dev.to', 'devto'],
  ['medium.com', 'medium'],
  ['stackoverflow.com', 'stackoverflow'],
];

export type SignupChannelSource = {
  /** `document.referrer` — the empty string when the browser sends none. */
  referrer: string;
  /** `window.location.search` of the entry page. */
  search: string;
  /** `window.location.hostname` — used to recognise our own pages. */
  host: string;
};

/**
 * Derive the channel of an entry page. Pure: everything it reads is an
 * argument, so it can be exercised without a DOM.
 */
export function deriveSignupChannel(source: SignupChannelSource): string {
  const params = new URLSearchParams(source.search);

  if (params.get('gclid') !== null) {
    return 'ads:google';
  }

  const campaign = slugify(params.get('utm_source'));
  if (campaign !== '') {
    return `campaign:${campaign}`;
  }

  if (source.referrer === '') {
    return DIRECT_CHANNEL;
  }

  if (source.referrer.length > MAX_REFERRER_LENGTH) {
    return UNKNOWN_CHANNEL;
  }

  const host = hostOf(source.referrer);
  if (host === '') {
    return UNKNOWN_CHANNEL;
  }

  // Our own pages say nothing about where the visitor came from — the answer
  // was on the page that led them to www.hook0.com, which this tab never saw.
  // Claiming `referral:www.hook0.com` would fill the column with a fact
  // everyone already knows and hide the ones nobody does.
  if (isOwnHost(host, source.host)) {
    return UNKNOWN_CHANNEL;
  }

  return classify(host);
}

/**
 * The slice of `Storage` this module needs. Taking it as an argument keeps the
 * decision logic testable against a real implementation instead of a stand-in
 * for a browser.
 */
export type SignupChannelStorage = {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
};

/**
 * `window.sessionStorage`, wrapped so that a browser which refuses storage
 * (private mode, blocked cookies) costs the channel rather than the page that
 * hosts the sign-up form.
 */
export function browserSessionStorage(): SignupChannelStorage {
  return {
    getItem: (key) => {
      try {
        return window.sessionStorage.getItem(key);
      } catch {
        return null;
      }
    },
    setItem: (key, value) => {
      try {
        window.sessionStorage.setItem(key, value);
      } catch {
        // Nothing to recover: the channel is lost for this tab, the page is not.
      }
    },
  };
}

/**
 * Record the channel of this tab's entry page, once. Called at bootstrap so
 * the referrer is still the external one — by the time the sign-up form is
 * open, the browser reports the previous in-app page instead.
 *
 * The first entry wins: a visitor who comes back to the form later in the same
 * tab keeps the origin that brought them in.
 */
export function rememberSignupChannel(
  source: SignupChannelSource,
  storage: SignupChannelStorage
): void {
  if (storage.getItem(SESSION_STORAGE_KEY) !== null) {
    return;
  }

  storage.setItem(SESSION_STORAGE_KEY, deriveSignupChannel(source));
}

/** The channel remembered for this tab, or `unknown` when none was recorded. */
export function readSignupChannel(storage: SignupChannelStorage): string {
  const stored = storage.getItem(SESSION_STORAGE_KEY);
  return stored === null ? UNKNOWN_CHANNEL : stored;
}

/** The entry page as the browser currently describes it. */
export function currentPageSource(): SignupChannelSource {
  return {
    referrer: document.referrer,
    search: window.location.search,
    host: window.location.hostname,
  };
}

function classify(host: string): string {
  const assistant = lookup(ASSISTANTS, host);
  if (assistant !== '') {
    return `ai:${assistant}`;
  }

  // Google answers from ~190 country domains (google.fr, google.co.uk, …), so
  // the leading label decides rather than a list of TLDs that would rot. It is
  // the leading label and not the registrable domain on purpose: what merely
  // lives under google.com is not the search engine — gemini.google.com is an
  // assistant, news.google.com is a site that linked to us.
  if (leadingLabel(host) === 'google') {
    return 'organic:google';
  }

  const engine = lookup(SEARCH_ENGINES, host);
  if (engine !== '') {
    return `organic:${engine}`;
  }

  const network = lookup(SOCIAL, host);
  if (network !== '') {
    return `social:${network}`;
  }

  const suffix = slugify(host);
  return suffix === '' ? UNKNOWN_CHANNEL : `referral:${suffix}`;
}

function lookup(table: ReadonlyArray<readonly [string, string]>, host: string): string {
  for (const [domain, channel] of table) {
    if (host === domain || host.endsWith(`.${domain}`)) {
      return channel;
    }
  }
  return '';
}

function hostOf(referrer: string): string {
  try {
    return stripWww(new URL(referrer).hostname.toLowerCase());
  } catch {
    return '';
  }
}

function isOwnHost(referrerHost: string, currentHost: string): boolean {
  const own = stripWww(currentHost.toLowerCase());
  return (
    referrerHost === own || referrerHost.endsWith('.hook0.com') || referrerHost === 'hook0.com'
  );
}

function stripWww(host: string): string {
  return host.startsWith('www.') ? host.slice(4) : host;
}

function leadingLabel(host: string): string {
  return host.split('.')[0];
}

/**
 * Reduce a value to what the shared vocabulary accepts after the `:` — a short
 * lowercase slug. Anything else yields the empty string, which every caller
 * turns into a label the API recognises rather than passing on.
 */
function slugify(value: string | null): string {
  if (value === null) {
    return '';
  }

  const slug = value.trim().toLowerCase();
  if (slug.length === 0 || slug.length > MAX_SUFFIX_LENGTH) {
    return '';
  }

  const isSlug = /^[a-z0-9][a-z0-9.-]*$/.test(slug);
  return isSlug ? slug : '';
}

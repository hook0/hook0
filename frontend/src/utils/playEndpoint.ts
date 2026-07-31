// Utilities for Hook0 Play (play.hook0.com) — a free, no-signup webhook test inbox.
// Kept free of `import.meta` so it stays unit-testable under ts-jest (node env);
// callers pass the base URL explicitly (see VITE_PLAY_ENDPOINT).

const BASE62 = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
const TOKEN_PREFIX = 'c_';
const TOKEN_RANDOM_LENGTH = 27;

// Generate a Play token matching the server format `c_` + 27 base62 chars
// (mirrors play/src/relay/token.rs). Uses the Web Crypto API for randomness.
export function generatePlayToken(): string {
  const bytes = new Uint8Array(TOKEN_RANDOM_LENGTH);
  globalThis.crypto.getRandomValues(bytes);
  let random = '';
  for (let i = 0; i < TOKEN_RANDOM_LENGTH; i++) {
    random += BASE62[bytes[i] % BASE62.length];
  }
  return TOKEN_PREFIX + random;
}

// Validate a token against the server's accepted format.
export function isValidPlayToken(token: string): boolean {
  if (!token.startsWith(TOKEN_PREFIX)) {
    return false;
  }
  const random = token.slice(TOKEN_PREFIX.length);
  if (random.length !== TOKEN_RANDOM_LENGTH) {
    return false;
  }
  return /^[0-9A-Za-z]+$/.test(random);
}

// Strip trailing slashes so URL segments join cleanly.
export function normalizePlayBaseUrl(base: string): string {
  return base.replace(/\/+$/, '');
}

// URL that receives webhooks for a token: {base}/in/{token}/
export function buildPlayReceiveUrl(base: string, token: string): string {
  return `${normalizePlayBaseUrl(base)}/in/${token}/`;
}

// Human-facing inspector page: {base}/view/{token}
export function buildPlayViewUrl(base: string, token: string): string {
  return `${normalizePlayBaseUrl(base)}/view/${token}`;
}

// Inspection API returning received webhooks: {base}/api/tokens/{token}/webhooks
export function buildPlayInspectUrl(base: string, token: string): string {
  return `${normalizePlayBaseUrl(base)}/api/tokens/${token}/webhooks`;
}

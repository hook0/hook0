/**
 * When a resend cooldown started, or the fact that none did.
 *
 * Modelled as a tagged union rather than a nullable number or a `0` sentinel:
 * "no cooldown yet" is a distinct case every caller has to handle, and no
 * timestamp value is overloaded to mean absence.
 */
export type CooldownStart = { readonly kind: 'started'; readonly atMs: number } | { kind: 'never' };

/** The single "no cooldown recorded" value. */
export const NO_COOLDOWN: CooldownStart = { kind: 'never' };

/**
 * Seconds left before a cooldown that started at `start` and lasts
 * `durationSeconds` is over, evaluated at `nowMs`.
 *
 * The result is clamped to `[0, durationSeconds]` and rounded up, so a
 * just-started cooldown reads as its full duration and the last value shown
 * before it re-enables is `1`. A cooldown that never started reads as `0`.
 */
export function remainingCooldownSeconds(
  start: CooldownStart,
  durationSeconds: number,
  nowMs: number
): number {
  if (start.kind === 'never') {
    return 0;
  }
  const remainingMs = durationSeconds * 1000 - (nowMs - start.atMs);
  if (remainingMs <= 0) {
    return 0;
  }
  return Math.min(durationSeconds, Math.ceil(remainingMs / 1000));
}

/**
 * The later of two cooldown starts — the one that still has time left when they
 * disagree. A cooldown that never started loses to any that did.
 *
 * Needed because a page can learn about a cooldown from two places at once: what
 * this browser recorded when the visitor last pressed the button, and what the
 * page that redirected here declared it had just sent. Taking the later of the
 * two means neither an old record nor a stale hand-off can re-enable the button
 * while the server is still refusing to send.
 */
export function latestCooldownStart(a: CooldownStart, b: CooldownStart): CooldownStart {
  if (a.kind === 'never') {
    return b;
  }
  if (b.kind === 'never') {
    return a;
  }
  return a.atMs >= b.atMs ? a : b;
}

/**
 * The subset of the Web Storage API this module needs. Narrow on purpose: it
 * makes the dependency explicit at the call site (which passes the real
 * `window.sessionStorage`) and keeps this module usable outside a browser.
 */
export interface CooldownStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

/**
 * Storage key holding the most recent resend cooldown.
 *
 * One key, not one per address: the stored record carries the address it belongs
 * to, so storage stays bounded to a single entry however many addresses a
 * visitor cycles through, while a cooldown recorded for one address is never
 * applied to another.
 */
const COOLDOWN_STORAGE_KEY = 'hook0.resend-verification-cooldown';

/**
 * Read back the cooldown recorded for `email`. Anything other than an intact
 * record for that exact address — nothing stored, stored for someone else,
 * unreadable or corrupt storage — reads as "never started", which at worst costs
 * the visitor one extra attempt the server then refuses.
 */
export function readCooldownStart(storage: CooldownStorage, email: string): CooldownStart {
  let raw: string | null;
  try {
    raw = storage.getItem(COOLDOWN_STORAGE_KEY);
  } catch {
    // Storage can be unavailable (private browsing, blocked storage): the
    // countdown is a courtesy, the cooldown that counts lives on the server.
    return NO_COOLDOWN;
  }
  if (raw === null) {
    return NO_COOLDOWN;
  }

  let parsed: { email: unknown; startedAtMs: unknown } | null;
  try {
    parsed = JSON.parse(raw) as typeof parsed;
  } catch {
    return NO_COOLDOWN;
  }

  if (
    parsed === null ||
    typeof parsed !== 'object' ||
    parsed.email !== email ||
    typeof parsed.startedAtMs !== 'number' ||
    !Number.isFinite(parsed.startedAtMs)
  ) {
    return NO_COOLDOWN;
  }

  return { kind: 'started', atMs: parsed.startedAtMs };
}

/**
 * Record that a cooldown for `email` started at `startedAtMs`, so it survives a
 * reload of the page. Silently gives up if storage refuses the write.
 */
export function writeCooldownStart(
  storage: CooldownStorage,
  email: string,
  startedAtMs: number
): void {
  try {
    storage.setItem(COOLDOWN_STORAGE_KEY, JSON.stringify({ email, startedAtMs }));
  } catch {
    // Same reasoning as reading: losing the record only re-enables the button
    // early, and the server still refuses to send.
  }
}

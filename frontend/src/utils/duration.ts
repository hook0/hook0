import parse from 'parse-duration';

const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 3600;
const SECONDS_PER_DAY = 86400;
export const SECONDS_PER_YEAR = 365 * SECONDS_PER_DAY;

export function formatDuration(seconds: number): string {
  if (seconds < SECONDS_PER_MINUTE) {
    return `${seconds}s`;
  }
  if (seconds < SECONDS_PER_HOUR) {
    return formatMinutes(seconds);
  }
  if (seconds < SECONDS_PER_DAY) {
    return formatHours(seconds);
  }
  if (seconds < SECONDS_PER_YEAR) {
    return formatDays(seconds);
  }
  return formatYears(seconds);
}

function formatMinutes(seconds: number): string {
  const mins = Math.floor(seconds / SECONDS_PER_MINUTE);
  const secs = seconds % SECONDS_PER_MINUTE;
  return secs > 0 ? `${mins}min ${secs}s` : `${mins}min`;
}

function formatHours(seconds: number): string {
  const hours = Math.floor(seconds / SECONDS_PER_HOUR);
  const mins = Math.floor((seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  return mins > 0 ? `${hours}h ${mins}min` : `${hours}h`;
}

function formatDays(seconds: number): string {
  const days = Math.floor(seconds / SECONDS_PER_DAY);
  const hours = Math.floor((seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR);
  return hours > 0 ? `${days}d ${hours}h` : `${days}d`;
}

function formatYears(seconds: number): string {
  const years = Math.floor(seconds / SECONDS_PER_YEAR);
  const days = Math.floor((seconds % SECONDS_PER_YEAR) / SECONDS_PER_DAY);
  return days > 0 ? `${years}y ${days}d` : `${years}y`;
}

// Bare-number path accepts only plain integers/decimals. Rejects scientific notation
// ('3e10'), hex ('0xFF'), infinities — all of which Number() would silently coerce.
const BARE_NUMBER = /^\d+(?:\.\d+)?$/;

/**
 * Parses a human-readable duration string (e.g. "1h30min", "5s", "2d") into total seconds.
 * Bare numbers are treated as raw seconds. Returns null for unparseable or negative input.
 *
 * @example
 * parseDuration('1h30min') // => 5400
 * parseDuration('5s')      // => 5
 * parseDuration('300')     // => 300
 * parseDuration('abc')     // => null
 * parseDuration('3e10')    // => null
 */
export function parseDuration(input: string): number | null {
  const trimmed = input.trim();
  if (!trimmed) {
    return null;
  }

  if (BARE_NUMBER.test(trimmed)) {
    return Math.round(Number(trimmed));
  }

  const result = parse(trimmed, 's');
  if (result === null || !Number.isFinite(result) || result < 0) {
    return null;
  }
  return Math.round(result);
}

/**
 * Parses a human-readable duration string (e.g. "1h30min", "5s", "2d") into total seconds.
 * Falls back to treating bare numbers as raw seconds. Returns null for unparseable input.
 *
 * @example
 * parseDuration('1h30min') // => 5400
 * parseDuration('5s')      // => 5
 * parseDuration('300')     // => 300
 * parseDuration('abc')     // => null
 */

const UNITS: Record<string, number> = {
  s: 1,
  sec: 1,
  min: 60,
  h: 3600,
  d: 86400,
};

const TOKEN_REGEX = /(\d+(?:\.\d+)?)\s*(d|h|min|sec|s)/gi;
// Only characters that can appear in a valid duration — digits, whitespace, decimal points, and unit letters (d, h, m, s, e, i, n, c)
const VALID_CHARS_REGEX = /^[\d\s.dhmseinc]+$/i;

export function parseDuration(input: string): number | null {
  const trimmed = input.trim();
  if (!trimmed) return null;

  // Bare number — interpret as raw seconds
  const asNumber = Number(trimmed);
  if (!Number.isNaN(asNumber) && asNumber >= 0) return Math.round(asNumber);

  // Reject strings with characters that can't belong to any duration token
  if (!VALID_CHARS_REGEX.test(trimmed)) return null;

  let total = 0;
  let matched = false;
  let totalMatchedLength = 0;
  let match: RegExpExecArray | null;

  TOKEN_REGEX.lastIndex = 0;
  while ((match = TOKEN_REGEX.exec(trimmed)) !== null) {
    const value = parseFloat(match[1]);
    const unit = match[2].toLowerCase();
    const multiplier = UNITS[unit];
    if (multiplier === undefined) return null;
    total += value * multiplier;
    totalMatchedLength += match[0].length;
    matched = true;
  }

  // Ensure the entire input was consumed — leftover non-whitespace means garbled input (e.g. "5min foo")
  const nonWhitespaceLength = trimmed.replace(/\s/g, '').length;
  if (!matched || totalMatchedLength < nonWhitespaceLength) return null;

  return Math.round(total);
}

import parse from 'parse-duration';

/**
 * Parses a human-readable duration string (e.g. "1h30min", "5s", "2d") into total seconds.
 * Bare numbers are treated as raw seconds. Returns null for unparseable or negative input.
 *
 * @example
 * parseDuration('1h30min') // => 5400
 * parseDuration('5s')      // => 5
 * parseDuration('300')     // => 300
 * parseDuration('abc')     // => null
 */
export function parseDuration(input: string): number | null {
  const trimmed = input.trim();
  if (!trimmed) return null;

  const asNumber = Number(trimmed);
  if (!Number.isNaN(asNumber) && asNumber >= 0) return Math.round(asNumber);

  const result = parse(trimmed, 's');
  if (result === null || Number.isNaN(result) || result < 0) return null;
  return Math.round(result);
}

const SENSITIVE_HEADERS = new Set([
  'cookie',
  'set-cookie',
  'authorization',
  'www-authenticate',
  'proxy-authorization',
  'proxy-authenticate',
]);

export function filterSensitiveHeaders(
  headers: Record<string, string> | undefined | null
): Record<string, string> | null {
  if (!headers) return null;
  const entries = Object.entries(headers)
    .filter(([key]) => !SENSITIVE_HEADERS.has(key.toLowerCase()))
    .sort(([a], [b]) => a.localeCompare(b));
  if (entries.length === 0) return null;
  return Object.fromEntries(entries);
}

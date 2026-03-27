const SENSITIVE_HEADERS = new Set([
  'cookie',
  'set-cookie',
  'authorization',
  'www-authenticate',
  'proxy-authorization',
  'proxy-authenticate',
]);

export function filterSensitiveHeaders(
  headers: Record<string, unknown> | undefined | null
): Record<string, unknown> | null {
  if (!headers) return null;
  const entries = Object.entries(headers).filter(
    ([key]) => !SENSITIVE_HEADERS.has(key.toLowerCase())
  );
  if (entries.length === 0) return null;
  return Object.fromEntries(entries);
}

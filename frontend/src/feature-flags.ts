// Build allowlist of trusted API endpoint origins from build-time config
const ANY_PORT_HOSTNAMES = new Set(['localhost', '127.0.0.1', '[::1]']);
const allowedApiOrigins: Set<string> = new Set();
const allowedAnyPortApiOrigins: Set<string> = new Set();

try {
  const defaultEndpoint = import.meta.env.VITE_API_ENDPOINT ?? '';
  if (defaultEndpoint) {
    allowedApiOrigins.add(new URL(defaultEndpoint).origin);
  }
} catch {
  // VITE_API_ENDPOINT is not a valid URL — allowlist will be empty (only explicitly allowed origins)
}
for (const entry of (import.meta.env.VITE_ALLOWED_API_ORIGINS ?? '').split(',')) {
  const trimmed = entry.trim();
  if (trimmed) {
    try {
      const parsed = new URL(trimmed);

      allowedApiOrigins.add(parsed.origin);
      if (ANY_PORT_HOSTNAMES.has(parsed.hostname) && parsed.port === '') {
        allowedAnyPortApiOrigins.add(parsed.protocol + '//' + parsed.hostname);
      }
    } catch {
      // skip invalid entries
    }
  }
}

function validateApiEndpoint(value: string): boolean {
  try {
    const url = new URL(value);

    if (url.protocol !== 'http:' && url.protocol !== 'https:') {
      return false;
    } else if (
      allowedApiOrigins.has(url.origin) ||
      allowedAnyPortApiOrigins.has(url.protocol + '//' + url.hostname)
    ) {
      return true;
    } else {
      return false;
    }
  } catch {
    return false;
  }
}

const queryParams: Record<string, string> = [...new URLSearchParams(location.search)].reduce(
  function toObj(o, pair) {
    const [k, v]: string[] = pair;
    // @ts-ignore
    o[k] = v;
    return o;
  },
  {}
);

// Sanitize API_ENDPOINT from query string: reject untrusted origins
if (
  Object.prototype.hasOwnProperty.call(queryParams, 'API_ENDPOINT') &&
  !validateApiEndpoint(queryParams['API_ENDPOINT'])
) {
  delete queryParams['API_ENDPOINT'];
  console.warn('Invalid API_ENDPOINT query string parameter; ignoring');
}

export default {
  getOrElse(feature: string, fallback: string): string {
    return Object.prototype.hasOwnProperty.call(queryParams, feature)
      ? queryParams[feature]
      : fallback;
  },
  getIntegerOrElse(feature: string, fallback: number): number {
    return Object.prototype.hasOwnProperty.call(queryParams, feature) &&
      !Number.isNaN(parseInt(queryParams[feature], 10))
      ? parseInt(queryParams[feature], 10)
      : fallback;
  },
};

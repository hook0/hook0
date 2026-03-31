export function statusCodeClass(code: number | undefined | null): string {
  if (code == null) return 'response-status--unknown';
  if (code >= 200 && code < 300) return 'response-status--success';
  if (code >= 300 && code < 400) return 'response-status--warning';
  return 'response-status--error';
}

export function statusCodeClass(code: number | undefined | null): string {
  if (code == null) return 'log-detail__status-badge--unknown';
  if (code >= 200 && code < 300) return 'log-detail__status-badge--success';
  if (code >= 300 && code < 400) return 'log-detail__status-badge--warning';
  return 'log-detail__status-badge--error';
}

const dateFmt = new Intl.DateTimeFormat(undefined, {
  day: 'numeric',
  month: 'short',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
});

/**
 * Format an ISO date string to a short locale display (e.g. "31 Mar, 14:13:12").
 * Returns "\u2014" for null, undefined, or unparseable values.
 */
export function formatDate(val: string | null | undefined): string {
  if (!val) return '\u2014';
  const date = new Date(val);
  if (Number.isNaN(date.getTime())) return '\u2014';
  return dateFmt.format(date);
}

/**
 * Format a future date string as a human-readable countdown (e.g. "5m", "1h30m").
 * Returns "<1m" when the date is in the past.
 */
export function formatRelativeTime(dateStr: string): string {
  const diff = new Date(dateStr).getTime() - Date.now();
  if (diff <= 0) return '<1m';
  const mins = Math.ceil(diff / 60000);
  if (mins < 60) return `${mins}min`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m > 0 ? `${h}h ${m}min` : `${h}h`;
}

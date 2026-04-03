const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 3600;
const SECONDS_PER_DAY = 86400;
const SECONDS_PER_YEAR = 365 * SECONDS_PER_DAY;

export function formatDuration(seconds: number): string {
  if (seconds < SECONDS_PER_MINUTE) {
    return `${seconds}s`;
  }
  if (seconds < SECONDS_PER_HOUR) {
    const mins = Math.floor(seconds / SECONDS_PER_MINUTE);
    const secs = seconds % SECONDS_PER_MINUTE;
    return secs > 0 ? `${mins}min ${secs}s` : `${mins}min`;
  }
  if (seconds < SECONDS_PER_DAY) {
    const hours = Math.floor(seconds / SECONDS_PER_HOUR);
    const mins = Math.floor((seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
    return mins > 0 ? `${hours}h ${mins}min` : `${hours}h`;
  }
  if (seconds < SECONDS_PER_YEAR) {
    const days = Math.floor(seconds / SECONDS_PER_DAY);
    const hours = Math.floor((seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR);
    return hours > 0 ? `${days}d ${hours}h` : `${days}d`;
  }
  const years = Math.floor(seconds / SECONDS_PER_YEAR);
  const days = Math.floor((seconds % SECONDS_PER_YEAR) / SECONDS_PER_DAY);
  return days > 0 ? `${years}y ${days}d` : `${years}y`;
}

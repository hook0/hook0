// Source of truth for increasing delays: output-worker/src/main.rs compute_next_retry_duration()
export const INCREASING_DELAYS = [3, 10, 180, 1800, 3600, 10800, 18000, 36000];

export function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}min`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86400)}d`;
}

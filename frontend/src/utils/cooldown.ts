/**
 * Seconds left before a cooldown that started at `startedAtMs` and lasts
 * `durationSeconds` is over, evaluated at `nowMs`.
 *
 * The result is clamped to `[0, durationSeconds]` and rounded up, so a
 * just-started cooldown reads as its full duration and the last value shown
 * before it re-enables is `1`. A never-started cooldown (`startedAtMs` far in
 * the past, e.g. `0`) naturally returns `0`.
 */
export function remainingCooldownSeconds(
  startedAtMs: number,
  durationSeconds: number,
  nowMs: number
): number {
  const remainingMs = durationSeconds * 1000 - (nowMs - startedAtMs);
  if (remainingMs <= 0) {
    return 0;
  }
  return Math.min(durationSeconds, Math.ceil(remainingMs / 1000));
}

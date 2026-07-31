import { remainingCooldownSeconds } from './cooldown';

describe('remainingCooldownSeconds', () => {
  const start = 1_000_000;

  it('reads as the full duration at the very start', () => {
    expect(remainingCooldownSeconds(start, 60, start)).toBe(60);
  });

  it('counts down as time passes', () => {
    expect(remainingCooldownSeconds(start, 60, start + 1_000)).toBe(59);
    expect(remainingCooldownSeconds(start, 60, start + 30_500)).toBe(30);
  });

  it('rounds up so the last visible value is 1', () => {
    expect(remainingCooldownSeconds(start, 60, start + 59_100)).toBe(1);
  });

  it('is 0 exactly when and after the cooldown elapses', () => {
    expect(remainingCooldownSeconds(start, 60, start + 60_000)).toBe(0);
    expect(remainingCooldownSeconds(start, 60, start + 120_000)).toBe(0);
  });

  it('is 0 for a never-started cooldown', () => {
    expect(remainingCooldownSeconds(0, 60, start)).toBe(0);
  });

  it('never exceeds the duration even with clock skew', () => {
    expect(remainingCooldownSeconds(start, 60, start - 5_000)).toBe(60);
  });
});

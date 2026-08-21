import { describe, expect, test } from '@jest/globals';
import fc from 'fast-check';

import { MAX_ATTEMPTS_CAP, RetryPolicy } from '../src/index';

/**
 * Invariants a retry schedule holds, whatever policy it was built from and whatever randomness it
 * was jittered with: a delay is never negative, never above the ceiling of its own retry, the
 * ceilings never shrink as retries pile up, there are never more delays than the policy allows
 * attempts, and they never add up to more than the budget they share.
 */

/** Fixed so a run explores the same cases on every machine, and cannot flake in CI. */
const RUNS = { seed: 20260813, numRuns: 512, verbose: false };

/** Retries a schedule is inspected over, well past what any policy is allowed to make. */
const INSPECTED_RETRIES = 64;

/**
 * A policy spanning what a caller can configure: no retry at all, more attempts than the cap
 * allows, a backoff larger than its own ceiling, and budgets from nothing to a minute.
 */
const aPolicy = fc
  .record({
    maxAttempts: fc.integer({ min: 0, max: 40 }),
    initialBackoffMs: fc.integer({ min: 0, max: 5_000 }),
    maxBackoffMs: fc.integer({ min: 0, max: 10_000 }),
    maxTotalDelayMs: fc.integer({ min: 0, max: 60_000 }),
  })
  .map(
    (fields) =>
      new RetryPolicy(
        fields.maxAttempts,
        fields.initialBackoffMs,
        fields.maxBackoffMs,
        fields.maxTotalDelayMs
      )
  );

/** One random draw per retry, plus the values a broken source of randomness could hand over. */
const someDraws = fc.array(
  fc.oneof(
    { weight: 90, arbitrary: fc.double({ min: 0, max: 1, noNaN: true }) },
    {
      weight: 10,
      arbitrary: fc.constantFrom(Number.NaN, Number.POSITIVE_INFINITY, -1, 2),
    }
  ),
  { maxLength: 48 }
);

describe('RetryPolicy', () => {
  test('makes between one attempt and the cap, whatever it was asked for', () => {
    fc.assert(
      fc.property(aPolicy, (policy) => {
        const attempts = policy.attempts();
        expect(attempts).toBeGreaterThanOrEqual(1);
        expect(attempts).toBeLessThanOrEqual(MAX_ATTEMPTS_CAP);
      }),
      RUNS
    );
  });

  test('never shrinks a ceiling and never passes the maximum backoff', () => {
    fc.assert(
      fc.property(aPolicy, (policy) => {
        for (let retry = 1; retry <= INSPECTED_RETRIES; retry += 1) {
          const ceiling = policy.backoffCeilingMs(retry);
          expect(ceiling).toBeGreaterThanOrEqual(0);
          expect(ceiling).toBeLessThanOrEqual(policy.maxBackoffMs);
          // Each delay is drawn uniformly between zero and its ceiling, so half of it on average:
          // ceilings that never shrink are what keeps the expected delay from shrinking either.
          expect(ceiling).toBeLessThanOrEqual(policy.backoffCeilingMs(retry + 1));
        }
      }),
      RUNS
    );
  });

  test('schedules delays that stay inside every bound of the policy', () => {
    fc.assert(
      fc.property(aPolicy, someDraws, (policy, draws) => {
        const delays = policy.delaysMs(draws);

        expect(delays.length).toBeLessThan(policy.attempts());

        let total = 0;
        delays.forEach((delay, index) => {
          expect(delay).toBeGreaterThanOrEqual(0);
          expect(delay).toBeLessThanOrEqual(policy.backoffCeilingMs(index + 1));
          total += delay;
        });

        expect(total).toBeLessThanOrEqual(policy.maxTotalDelayMs);
      }),
      RUNS
    );
  });

  test('schedules nothing to wait for when retrying is switched off', () => {
    fc.assert(
      fc.property(someDraws, (draws) => {
        const policy = RetryPolicy.disabled();
        expect(policy.attempts()).toStrictEqual(1);
        expect(policy.delaysMs(draws)).toStrictEqual([]);
      }),
      RUNS
    );
  });
});

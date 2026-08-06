import {
  MONETIZATION_CATEGORY,
  QUOTA_THRESHOLD_DANGER,
  QUOTA_THRESHOLD_WARNING,
  isQuotaNearLimit,
  quotaPercent,
  quotaSeverity,
  quotaWarningNames,
  type TrackedQuota,
} from './monetizationTracking';
import { UNLIMITED_QUOTA } from '../constants';

describe('monetizationTracking', () => {
  it('exposes a stable Matomo category', () => {
    expect(MONETIZATION_CATEGORY).toBe('monetization');
  });

  describe('quotaPercent', () => {
    it('reads unlimited limits as 0%', () => {
      expect(quotaPercent(1000, UNLIMITED_QUOTA)).toBe(0);
    });

    it('reads a zero limit as 0%', () => {
      expect(quotaPercent(5, 0)).toBe(0);
    });

    it('reads a negative limit as 0%', () => {
      expect(quotaPercent(5, -10)).toBe(0);
    });

    it('rounds the ratio for a normal limit', () => {
      expect(quotaPercent(1, 3)).toBe(33);
      expect(quotaPercent(2, 3)).toBe(67);
      expect(quotaPercent(9, 10)).toBe(90);
      expect(quotaPercent(10, 10)).toBe(100);
    });
  });

  describe('quotaSeverity', () => {
    it('is ok below the warning threshold', () => {
      expect(quotaSeverity(0, 10)).toBe('ok');
      expect(quotaSeverity(QUOTA_THRESHOLD_WARNING - 1, 100)).toBe('ok');
    });

    it('is warning between the warning and danger thresholds', () => {
      expect(quotaSeverity(QUOTA_THRESHOLD_WARNING, 100)).toBe('warning');
      expect(quotaSeverity(QUOTA_THRESHOLD_DANGER - 1, 100)).toBe('warning');
    });

    it('is danger at or above the danger threshold', () => {
      expect(quotaSeverity(QUOTA_THRESHOLD_DANGER, 100)).toBe('danger');
      expect(quotaSeverity(120, 100)).toBe('danger');
    });
  });

  describe('isQuotaNearLimit', () => {
    it('ignores flat display rows even when the numbers look maxed', () => {
      const retention: TrackedQuota = {
        name: 'Retention',
        consumption: 7,
        quota: 7,
        displayValue: '7',
      };
      expect(isQuotaNearLimit(retention)).toBe(false);
    });

    it('is false when a bar quota has headroom', () => {
      expect(isQuotaNearLimit({ name: 'Members', consumption: 1, quota: 10 })).toBe(false);
    });

    it('is true when a bar quota is near its limit', () => {
      expect(isQuotaNearLimit({ name: 'Members', consumption: 7, quota: 10 })).toBe(true);
    });

    it('is true when a bar quota is at its limit', () => {
      expect(isQuotaNearLimit({ name: 'Applications', consumption: 10, quota: 10 })).toBe(true);
    });
  });

  describe('quotaWarningNames', () => {
    it('returns an empty list when nothing is near a limit', () => {
      expect(
        quotaWarningNames([
          { name: 'Members', consumption: 1, quota: 10 },
          { name: 'Applications', consumption: 0, quota: 5 },
        ])
      ).toEqual([]);
    });

    it('keeps only near-limit bar quotas, in input order', () => {
      const quotas: TrackedQuota[] = [
        { name: 'Members', consumption: 9, quota: 10 },
        { name: 'Applications', consumption: 1, quota: 5 },
        { name: 'Retention', consumption: 7, quota: 7, displayValue: '7' },
        { name: 'Events', consumption: 70, quota: 100 },
      ];
      expect(quotaWarningNames(quotas)).toEqual(['Members', 'Events']);
    });
  });
});

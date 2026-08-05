// Analytics decisions for the monetization nudges shown on the dashboards
// (upgrade card, quota cards, near-limit hint). Kept free of `@/` aliases and
// Vue imports so it stays unit-testable under ts-jest (node env). The severity
// thresholds mirror the gauge in Hook0Consumption.vue so a tracked "warning"
// always matches what the user actually sees.

import { UNLIMITED_QUOTA } from '../constants';

/** Matomo event category shared by every monetization touchpoint. */
export const MONETIZATION_CATEGORY = 'monetization';

/** Consumption ratio (%) at which the gauge turns amber. */
export const QUOTA_THRESHOLD_WARNING = 70;
/** Consumption ratio (%) at which the gauge turns red. */
export const QUOTA_THRESHOLD_DANGER = 90;

export type QuotaSeverity = 'ok' | 'warning' | 'danger';

/** Minimal shape the tracking decisions read from a consumption row. */
export interface TrackedQuota {
  name: string;
  consumption: number;
  quota: number;
  /** When set, the gauge shows a flat value instead of a bar (e.g. retention). */
  displayValue?: string;
}

/**
 * Percentage of a quota consumed. Unlimited or unknown limits read as 0%,
 * matching the gauge (Hook0Consumption.vue).
 */
export function quotaPercent(consumption: number, quota: number): number {
  if (quota >= UNLIMITED_QUOTA || quota <= 0) {
    return 0;
  }
  return Math.round((consumption / quota) * 100);
}

/** Severity bucket for a quota, using the same thresholds as the gauge. */
export function quotaSeverity(consumption: number, quota: number): QuotaSeverity {
  const pct = quotaPercent(consumption, quota);
  if (pct >= QUOTA_THRESHOLD_DANGER) {
    return 'danger';
  }
  if (pct >= QUOTA_THRESHOLD_WARNING) {
    return 'warning';
  }
  return 'ok';
}

/**
 * True when a quota sits at or above the warning threshold AND the gauge draws
 * it as a bar. Rows with a `displayValue` render a flat number (no bar, so no
 * warning colour to surface), mirroring `v-if="!quota.displayValue"` in the gauge.
 */
export function isQuotaNearLimit(quota: TrackedQuota): boolean {
  if (quota.displayValue) {
    return false;
  }
  return quotaSeverity(quota.consumption, quota.quota) !== 'ok';
}

/**
 * Names of the quotas worth nudging on (near or at their limit). Empty when the
 * user has headroom everywhere. Order follows the input, so the first entry is
 * the one to feature in a single inline hint.
 */
export function quotaWarningNames(quotas: readonly TrackedQuota[]): string[] {
  return quotas.filter(isQuotaNearLimit).map((quota) => quota.name);
}

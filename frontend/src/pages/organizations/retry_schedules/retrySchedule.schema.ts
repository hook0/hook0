import { z } from 'zod';
import i18n from '@/plugins/i18n';
import type { RetryScheduleLimits } from './retrySchedule.types';

/**
 * Build a discriminated Zod schema from runtime limits published by /instance.
 * Must be a factory (not a module-level const) because limits come from an async query.
 */
export function makeRetryScheduleSchema(limits: RetryScheduleLimits) {
  const t = i18n.global.t;

  const name = z
    .string()
    .trim()
    .min(1, t('validation.required', { field: t('retrySchedules.fields.name') }))
    .max(200, t('retrySchedules.validation.nameTooLong'));

  const exponential = z.object({
    strategy: z.literal('exponential_increasing'),
    name,
    max_retries: z
      .number()
      .int()
      .min(1)
      .max(
        limits.max_retries,
        t('retrySchedules.validation.maxRetries', { max: limits.max_retries })
      ),
    base_delay: z
      .number()
      .int()
      .min(limits.exponential_base_delay_min_secs)
      .max(limits.exponential_base_delay_max_secs),
    wait_factor: z
      .number()
      .min(limits.exponential_wait_factor_min)
      .max(limits.exponential_wait_factor_max),
  });

  const linear = z.object({
    strategy: z.literal('linear'),
    name,
    max_retries: z.number().int().min(1).max(limits.max_retries),
    delay: z.number().int().min(limits.min_single_delay_secs).max(limits.max_single_delay_secs),
  });

  const custom = z.object({
    strategy: z.literal('custom'),
    name,
    intervals: z
      .array(z.number().int().min(limits.min_single_delay_secs).max(limits.max_single_delay_secs))
      .min(1)
      .max(limits.max_custom_intervals_length),
  });

  return z
    .discriminatedUnion('strategy', [exponential, linear, custom])
    .superRefine((data, ctx) => {
      const total = estimateTotalDuration(data, limits);
      if (total > limits.max_total_duration_secs) {
        ctx.addIssue({
          code: 'custom',
          path: [],
          message: t('retrySchedules.validation.totalTooLong', {
            max: limits.max_total_duration_secs,
          }),
        });
      }
    });
}

export type RetryScheduleFormValues = z.infer<ReturnType<typeof makeRetryScheduleSchema>>;

function estimateTotalDuration(
  data: { strategy: string } & Record<string, unknown>,
  limits: RetryScheduleLimits
): number {
  switch (data.strategy) {
    case 'exponential_increasing': {
      const max = Number(data.max_retries);
      const base = Number(data.base_delay);
      const factor = Number(data.wait_factor);
      let sum = 0;
      for (let i = 0; i < max; i += 1) {
        sum += Math.min(base * Math.pow(factor, i), limits.max_single_delay_secs);
      }
      return sum;
    }
    case 'linear':
      return Number(data.max_retries) * Number(data.delay);
    case 'custom': {
      const intervals = Array.isArray(data.intervals) ? (data.intervals as number[]) : [];
      return intervals.reduce((acc, v) => acc + Math.min(v, limits.max_single_delay_secs), 0);
    }
    default:
      return 0;
  }
}

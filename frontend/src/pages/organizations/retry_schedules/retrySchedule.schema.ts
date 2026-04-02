import { z } from 'zod';
import i18n from '@/plugins/i18n';

const MAX_INTERVAL_SECONDS = 604800; // 1 week

export function createRetryScheduleSchema() {
  const t = i18n.global.t;

  return z
    .object({
      name: z
        .string()
        .min(2, t('validation.minLength', { field: t('retrySchedules.fields.name'), min: 2 }))
        .max(200),
      strategy: z.enum(['increasing', 'linear', 'custom']),
      max_retries: z.coerce.number().int().min(1).max(25),
      linear_delay: z.coerce.number().int().min(1).max(MAX_INTERVAL_SECONDS).optional().nullable(),
      custom_intervals: z
        .array(z.coerce.number().int().min(1).max(MAX_INTERVAL_SECONDS))
        .optional()
        .nullable(),
      increasing_base_delay: z.coerce.number().int().min(1).max(3600).optional().nullable(),
      increasing_wait_factor: z.coerce.number().min(1.5).max(10).optional().nullable(),
    })
    .superRefine((data, ctx) => {
      if (data.strategy === 'increasing') {
        if (!data.increasing_base_delay) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: t('validation.required', {
              field: t('retrySchedules.fields.increasingBaseDelay'),
            }),
            path: ['increasing_base_delay'],
          });
        }
        if (!data.increasing_wait_factor) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: t('validation.required', {
              field: t('retrySchedules.fields.increasingWaitFactor'),
            }),
            path: ['increasing_wait_factor'],
          });
        }
      }
      if (data.strategy === 'linear' && !data.linear_delay) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          message: t('validation.required', {
            field: t('retrySchedules.fields.linearDelay'),
          }),
          path: ['linear_delay'],
        });
      }
      if (data.strategy === 'custom') {
        if (!data.custom_intervals || data.custom_intervals.length === 0) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: t('validation.required', {
              field: t('retrySchedules.fields.customIntervals'),
            }),
            path: ['custom_intervals'],
          });
        } else if (data.custom_intervals.length !== data.max_retries) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: t('validation.arrayLength', {
              field: t('retrySchedules.fields.customIntervals'),
              length: data.max_retries,
            }),
            path: ['custom_intervals'],
          });
        }
      }
    });
}

export type RetryScheduleFormValues = z.infer<ReturnType<typeof createRetryScheduleSchema>>;

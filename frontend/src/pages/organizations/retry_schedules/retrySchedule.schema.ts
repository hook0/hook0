import { z } from 'zod';
import i18n from '@/plugins/i18n';

const MAX_INTERVAL_SECONDS = 604800; // 1 week
const MIN_NAME_LENGTH = 2;
const MAX_NAME_LENGTH = 200;
const MAX_RETRIES = 25;
const MAX_BASE_DELAY = 3600;
const MIN_WAIT_FACTOR = 1.5;
const MAX_WAIT_FACTOR = 10;

type SchemaData = {
  strategy: 'increasing' | 'linear' | 'custom';
  max_retries: number;
  linear_delay?: number | null;
  custom_intervals?: number[] | null;
  increasing_base_delay?: number | null;
  increasing_wait_factor?: number | null;
};

type TranslateFn = typeof i18n.global.t;

function validateIncreasingStrategy(data: SchemaData, ctx: z.RefinementCtx, t: TranslateFn): void {
  if (data.increasing_base_delay === null || data.increasing_base_delay === undefined) {
    ctx.addIssue({
      code: 'custom',
      message: t('validation.required', {
        field: t('retrySchedules.fields.increasingBaseDelay'),
      }),
      path: ['increasing_base_delay'],
    });
  }
  if (data.increasing_wait_factor === null || data.increasing_wait_factor === undefined) {
    ctx.addIssue({
      code: 'custom',
      message: t('validation.required', {
        field: t('retrySchedules.fields.increasingWaitFactor'),
      }),
      path: ['increasing_wait_factor'],
    });
  }
}

function validateLinearStrategy(data: SchemaData, ctx: z.RefinementCtx, t: TranslateFn): void {
  if (data.linear_delay === null || data.linear_delay === undefined) {
    ctx.addIssue({
      code: 'custom',
      message: t('validation.required', {
        field: t('retrySchedules.fields.linearDelay'),
      }),
      path: ['linear_delay'],
    });
  }
}

function validateCustomStrategy(data: SchemaData, ctx: z.RefinementCtx, t: TranslateFn): void {
  if (!data.custom_intervals || data.custom_intervals.length === 0) {
    ctx.addIssue({
      code: 'custom',
      message: t('validation.required', {
        field: t('retrySchedules.fields.customIntervals'),
      }),
      path: ['custom_intervals'],
    });
  } else if (data.custom_intervals.length !== data.max_retries) {
    ctx.addIssue({
      code: 'custom',
      message: t('validation.arrayLength', {
        field: t('retrySchedules.fields.customIntervals'),
        length: data.max_retries,
      }),
      path: ['custom_intervals'],
    });
  }
}

export function createRetryScheduleSchema() {
  const t = i18n.global.t;

  return z
    .object({
      name: z
        .string()
        .min(
          MIN_NAME_LENGTH,
          t('validation.minLength', {
            field: t('retrySchedules.fields.name'),
            min: MIN_NAME_LENGTH,
          })
        )
        .max(MAX_NAME_LENGTH),
      strategy: z.enum(['increasing', 'linear', 'custom']),
      max_retries: z.coerce.number().int().min(1).max(MAX_RETRIES),
      linear_delay: z.coerce.number().int().min(1).max(MAX_INTERVAL_SECONDS).optional().nullable(),
      custom_intervals: z
        .array(z.coerce.number().int().min(1).max(MAX_INTERVAL_SECONDS))
        .optional()
        .nullable(),
      increasing_base_delay: z.coerce
        .number()
        .int()
        .min(1)
        .max(MAX_BASE_DELAY)
        .optional()
        .nullable(),
      increasing_wait_factor: z.coerce
        .number()
        .min(MIN_WAIT_FACTOR)
        .max(MAX_WAIT_FACTOR)
        .optional()
        .nullable(),
    })
    .superRefine((data, ctx) => {
      if (data.strategy === 'increasing') {
        validateIncreasingStrategy(data, ctx, t);
      }
      if (data.strategy === 'linear') {
        validateLinearStrategy(data, ctx, t);
      }
      if (data.strategy === 'custom') {
        validateCustomStrategy(data, ctx, t);
      }
    });
}

export type RetryScheduleFormValues = z.infer<ReturnType<typeof createRetryScheduleSchema>>;

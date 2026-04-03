// Zod validation schema for the retry schedule create/edit form.
//
// How it works:
// 1. baseSchema defines field-level types and ranges (shared between schema and type inference)
// 2. createRetryScheduleSchema() adds i18n error messages and cross-field validation via superRefine
// 3. superRefine dispatches to per-strategy validators that enforce "required when active" rules
// 4. Constants are exported for the UI (slider bounds) and validation messages

import { z } from 'zod';
import i18n from '@/plugins/i18n';

// These constants mirror the backend validation in api/src/handlers/retry_schedules.rs.
// If you change a value here, update the Rust counterpart too (and vice versa).
export const MAX_INTERVAL_SECONDS = 604800; // backend: MAX_INTERVAL_SECS
export const MIN_NAME_LENGTH = 2; // backend: #[validate(length(min = 2))] on RetrySchedulePost.name
export const MAX_NAME_LENGTH = 200; // backend: #[validate(length(max = 200))] on RetrySchedulePost.name
export const MAX_RETRIES = 25; // backend: max_retries range(min = 1, max = 25)
export const MAX_BASE_DELAY = 3600; // backend: MAX_BASE_DELAY_SECS
export const MIN_WAIT_FACTOR = 1.5; // backend: MIN_WAIT_FACTOR
export const MAX_WAIT_FACTOR = 10; // backend: MAX_WAIT_FACTOR
export const SLIDER_MAX_BASE_DELAY = 300; // UX cap — backend allows up to MAX_BASE_DELAY
export const SLIDER_MAX_LINEAR_DELAY = 86400; // UX cap — backend allows up to MAX_INTERVAL_SECONDS

const baseSchema = z.object({
  name: z.string().min(MIN_NAME_LENGTH).max(MAX_NAME_LENGTH),
  strategy: z.enum(['increasing', 'linear', 'custom']),
  max_retries: z.coerce.number().int().min(1).max(MAX_RETRIES),
  linear_delay: z.coerce.number().int().min(1).max(MAX_INTERVAL_SECONDS).optional().nullable(),
  custom_intervals: z
    .array(z.coerce.number().int().min(1).max(MAX_INTERVAL_SECONDS))
    .optional()
    .nullable(),
  increasing_base_delay: z.coerce.number().int().min(1).max(MAX_BASE_DELAY).optional().nullable(),
  increasing_wait_factor: z.coerce
    .number()
    .min(MIN_WAIT_FACTOR)
    .max(MAX_WAIT_FACTOR)
    .optional()
    .nullable(),
});

type SchemaData = z.infer<typeof baseSchema>;

type TranslateFn = typeof i18n.global.t;

// Increasing requires both base_delay and wait_factor — adds field-level errors so VeeValidate highlights the right input
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

/**
 * Builds the Zod schema with i18n-aware error messages.
 * Must be called at form setup (not module scope) because i18n.global.t reads the current locale.
 *
 * @example
 * const schema = createRetryScheduleSchema()
 * schema.parse({ name: 'Fast', strategy: 'linear', max_retries: 5, linear_delay: 60 })
 * // => validated RetryScheduleFormValues
 */
export function createRetryScheduleSchema() {
  const t = i18n.global.t;

  return (
    baseSchema
      .extend({
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
      })
      // superRefine runs after per-field validation — it's Zod's hook for cross-field rules that depend on the strategy discriminator
      .superRefine((data, ctx) => {
        switch (data.strategy) {
          case 'increasing':
            validateIncreasingStrategy(data, ctx, t);
            break;
          case 'linear':
            validateLinearStrategy(data, ctx, t);
            break;
          case 'custom':
            validateCustomStrategy(data, ctx, t);
            break;
        }
      })
  );
}

export type RetryScheduleFormValues = z.infer<ReturnType<typeof createRetryScheduleSchema>>;

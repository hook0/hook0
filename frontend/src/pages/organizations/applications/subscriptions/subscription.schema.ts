import { z } from 'zod';
import i18n from '@/plugins/i18n';

export function createSubscriptionSchema() {
  const t = i18n.global.t;
  return z.object({
    description: z
      .string()
      .min(1, t('validation.required', { field: t('fields.subscriptionDescription') })),
    target_method: z.string().min(1, t('validation.required', { field: t('fields.httpMethod') })),
    target_url: z
      .string()
      .min(1, t('validation.required', { field: t('fields.url') }))
      .url(t('validation.validUrl')),
  });
}

export type SubscriptionFormValues = z.infer<ReturnType<typeof createSubscriptionSchema>>;

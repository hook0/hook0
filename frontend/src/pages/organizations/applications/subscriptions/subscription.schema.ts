import { z } from 'zod';
import i18n from '@/plugins/i18n';

const t = i18n.global.t;

export const subscriptionSchema = z.object({
  description: z.string().min(1, t('validation.required', { field: 'Subscription description' })),
  target_method: z.string().min(1, t('validation.required', { field: 'HTTP method' })),
  target_url: z
    .string()
    .min(1, t('validation.required', { field: 'URL' }))
    .url(t('validation.validUrl')),
});

export type SubscriptionFormValues = z.infer<typeof subscriptionSchema>;

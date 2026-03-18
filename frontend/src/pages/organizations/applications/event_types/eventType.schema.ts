import { z } from 'zod';
import i18n from '@/plugins/i18n';

const t = i18n.global.t;

export const eventTypeSchema = z.object({
  service: z.string().min(1, t('validation.required', { field: 'Service' })),
  resource_type: z.string().min(1, t('validation.required', { field: 'Resource type' })),
  verb: z.string().min(1, t('validation.required', { field: 'Verb' })),
});

export type EventTypeFormValues = z.infer<typeof eventTypeSchema>;

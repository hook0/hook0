import { z } from 'zod';
import i18n from '@/plugins/i18n';

export function createEventTypeSchema() {
  const t = i18n.global.t;
  return z.object({
    service: z.string().min(1, t('validation.required', { field: t('fields.service') })),
    resource_type: z.string().min(1, t('validation.required', { field: t('fields.resourceType') })),
    verb: z.string().min(1, t('validation.required', { field: t('fields.verb') })),
  });
}

export type EventTypeFormValues = z.infer<ReturnType<typeof createEventTypeSchema>>;

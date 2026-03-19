import { z } from 'zod';
import i18n from '@/plugins/i18n';

export function createApplicationSchema() {
  const t = i18n.global.t;
  return z.object({
    name: z.string().min(1, t('validation.required', { field: t('fields.applicationName') })),
  });
}

export type ApplicationFormValues = z.infer<ReturnType<typeof createApplicationSchema>>;

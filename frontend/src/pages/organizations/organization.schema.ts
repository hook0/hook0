import { z } from 'zod';
import i18n from '@/plugins/i18n';

export function createOrganizationSchema() {
  const t = i18n.global.t;
  return z.object({
    name: z.string().min(1, t('validation.required', { field: t('fields.organizationName') })),
  });
}

export type OrganizationFormValues = z.infer<ReturnType<typeof createOrganizationSchema>>;

import { z } from 'zod';
import i18n from '@/plugins/i18n';

const t = i18n.global.t;

export const organizationSchema = z.object({
  name: z.string().min(1, t('validation.required', { field: 'Organization name' })),
});

export type OrganizationFormValues = z.infer<typeof organizationSchema>;

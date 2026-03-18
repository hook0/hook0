import { z } from 'zod';
import i18n from '@/plugins/i18n';

const t = i18n.global.t;

export const applicationSchema = z.object({
  name: z.string().min(1, t('validation.required', { field: 'Application name' })),
});

export type ApplicationFormValues = z.infer<typeof applicationSchema>;

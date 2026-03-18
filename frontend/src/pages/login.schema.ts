import { z } from 'zod';
import i18n from '@/plugins/i18n';

const t = i18n.global.t;

export const loginSchema = z.object({
  email: z.email(t('validation.validEmail')),
  password: z.string().min(1, t('validation.required', { field: 'Password' })),
});

export type LoginFormValues = z.infer<typeof loginSchema>;

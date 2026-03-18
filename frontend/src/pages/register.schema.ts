import { z } from 'zod';
import i18n from '@/plugins/i18n';

const t = i18n.global.t;

export const registerSchema = z.object({
  email: z.email(t('validation.validEmail')),
  firstName: z.string().min(1, t('validation.required', { field: 'First name' })),
  lastName: z.string().min(1, t('validation.required', { field: 'Last name' })),
  password: z.string().min(8, t('validation.passwordMinLength')),
});

export type RegisterFormValues = z.infer<typeof registerSchema>;

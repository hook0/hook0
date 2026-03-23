import { z } from 'zod';
import i18n from '@/plugins/i18n';

export function createRegisterSchema() {
  const t = i18n.global.t;
  return z.object({
    email: z.email(t('validation.validEmail')),
    firstName: z.string().min(1, t('validation.required', { field: t('fields.firstName') })),
    lastName: z.string().min(1, t('validation.required', { field: t('fields.lastName') })),
    password: z.string().min(8, t('validation.passwordMinLength')),
  });
}

export type RegisterFormValues = z.infer<ReturnType<typeof createRegisterSchema>>;

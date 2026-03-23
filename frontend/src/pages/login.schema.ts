import { z } from 'zod';
import i18n from '@/plugins/i18n';

export function createLoginSchema() {
  const t = i18n.global.t;
  return z.object({
    email: z.email(t('validation.validEmail')),
    password: z.string().min(1, t('validation.required', { field: t('fields.password') })),
  });
}

export type LoginFormValues = z.infer<ReturnType<typeof createLoginSchema>>;

import { z } from 'zod';
import i18n from '@/plugins/i18n';

export function createPasswordChangeSchema() {
  const t = i18n.global.t;
  return z
    .object({
      new_password: z.string().min(8, t('validation.passwordMinLength')),
      confirm_new_password: z.string().min(1, t('validation.passwordConfirm')),
    })
    .refine((data) => data.new_password === data.confirm_new_password, {
      message: t('validation.passwordsMismatch'),
      path: ['confirm_new_password'],
    });
}

export type PasswordChangeFormValues = z.infer<ReturnType<typeof createPasswordChangeSchema>>;

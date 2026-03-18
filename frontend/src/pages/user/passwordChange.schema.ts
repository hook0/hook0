import { z } from 'zod';
import i18n from '@/plugins/i18n';

const t = i18n.global.t;

export const passwordChangeSchema = z
  .object({
    new_password: z.string().min(8, t('validation.passwordMinLength')),
    confirm_new_password: z.string().min(1, t('validation.passwordConfirm')),
  })
  .refine((data) => data.new_password === data.confirm_new_password, {
    message: t('validation.passwordsMismatch'),
    path: ['confirm_new_password'],
  });

export type PasswordChangeFormValues = z.infer<typeof passwordChangeSchema>;

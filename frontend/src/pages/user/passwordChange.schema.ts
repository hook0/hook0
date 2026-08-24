import { z } from 'zod';
import i18n from '@/plugins/i18n';
import {
  PASSWORD_MAXIMUM_LENGTH,
  WEAKNESS_MESSAGE_KEYS,
  checkPassword,
  type UserIdentity,
} from '@/utils/passwordPolicy';

/** See `createRegisterSchema`: the floor is instance configuration, not a literal. */
export function createPasswordChangeSchema(identity: UserIdentity, minimumLength: number) {
  const t = i18n.global.t;
  return (
    z
      .object({
        // The account proves it is still the one at the keyboard. Only its
        // presence is checked here: the policy that shapes a *new* password
        // says nothing about one chosen years ago, and the ceiling is the same
        // one the API enforced when it was set.
        current_password: z
          .string()
          .min(1, t('validation.required', { field: t('fields.currentPassword') }))
          .max(
            PASSWORD_MAXIMUM_LENGTH,
            t('validation.passwordMaxLength', { count: PASSWORD_MAXIMUM_LENGTH })
          ),
        new_password: z
          .string()
          .min(minimumLength, t('validation.passwordMinLength', { count: minimumLength }))
          .max(
            PASSWORD_MAXIMUM_LENGTH,
            t('validation.passwordMaxLength', { count: PASSWORD_MAXIMUM_LENGTH })
          ),
        confirm_new_password: z.string().min(1, t('validation.passwordConfirm')),
      })
      .refine((data) => data.new_password === data.confirm_new_password, {
        message: t('validation.passwordsMismatch'),
        path: ['confirm_new_password'],
      })
      // Same rule as registration: the API refuses a password built from the
      // account's own email address or name, so say it before the round trip.
      .superRefine((data, context) => {
        const verdict = checkPassword(data.new_password, identity);

        if (!verdict.acceptable) {
          context.addIssue({
            code: 'custom',
            path: ['new_password'],
            message: t(WEAKNESS_MESSAGE_KEYS[verdict.weakness]),
          });
        }
      })
  );
}

export type PasswordChangeFormValues = z.infer<ReturnType<typeof createPasswordChangeSchema>>;

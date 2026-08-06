import { z } from 'zod';
import i18n from '@/plugins/i18n';
import { WEAKNESS_MESSAGE_KEYS, checkPassword } from '@/utils/passwordPolicy';

/**
 * `minimumLength` comes from the instance (GET /instance), not from a number
 * written here: it is operator configuration, and a form that guesses it
 * either refuses passwords the API accepts or promises ones it refuses.
 */
export function createRegisterSchema(minimumLength: number) {
  const t = i18n.global.t;
  return (
    z
      .object({
        email: z.email(t('validation.validEmail')),
        firstName: z.string().min(1, t('validation.required', { field: t('fields.firstName') })),
        lastName: z.string().min(1, t('validation.required', { field: t('fields.lastName') })),
        password: z
          .string()
          .min(minimumLength, t('validation.passwordMinLength', { count: minimumLength })),
      })
      // The API refuses a password built from the account's own email address or
      // name; say so while the user is still typing rather than after a round
      // trip. The API keeps the final word, and the rest of the policy.
      .superRefine((values, context) => {
        const verdict = checkPassword(values.password, {
          email: values.email,
          firstName: values.firstName,
          lastName: values.lastName,
        });

        if (!verdict.acceptable) {
          context.addIssue({
            code: 'custom',
            path: ['password'],
            message: t(WEAKNESS_MESSAGE_KEYS[verdict.weakness]),
          });
        }
      })
  );
}

export type RegisterFormValues = z.infer<ReturnType<typeof createRegisterSchema>>;

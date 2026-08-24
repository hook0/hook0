import {
  isCurrentPasswordRefused,
  passwordRejection,
  validationRejection,
  type PasswordRejection,
} from '@/utils/passwordProblem';

/**
 * What a field has to say about a failed request: either nothing, or the one
 * line the user has to read to fix it.
 */
export type FieldMessage =
  { readonly shown: false } | { readonly shown: true; readonly message: string };

const SAYS_NOTHING: FieldMessage = { shown: false };

/**
 * Where a refused password change belongs on screen.
 *
 * `POST /auth/password` carries two passwords and the errors it answers with
 * name at most one of them, so the two fields are decided separately rather
 * than by whichever check runs first:
 *
 * - the policy's own errors carry no field, and the policy runs on the new
 *   password alone — they belong there, never under the one that only proved
 *   who was asking. Asking about the current password first is what used to
 *   put "this password appears in lists of commonly used passwords" under
 *   "Current password" and send the user back to retype one that was right;
 * - a generic `Validation` 422 does name its field, and can name either — a
 *   control character pasted out of a password manager lands on whichever
 *   field it was pasted into;
 * - `Forbidden` is the one refusal that is about the current password, and
 *   its wording comes from the caller: the API's own is deliberately vague
 *   and would only puzzle someone whose whole problem is a password they can
 *   retype.
 */
export interface ChangePasswordFailure {
  readonly currentPassword: FieldMessage;
  readonly newPassword: FieldMessage;
  /** Nothing on the form explains this failure, so something else has to. */
  readonly unexplained: boolean;
}

function messageOf(rejection: PasswordRejection): FieldMessage {
  if (!rejection.refused) {
    return SAYS_NOTHING;
  }
  return { shown: true, message: rejection.reason };
}

export function changePasswordFailure(
  error: unknown,
  currentPasswordRefusedMessage: string
): ChangePasswordFailure {
  if (isCurrentPasswordRefused(error)) {
    return {
      currentPassword: { shown: true, message: currentPasswordRefusedMessage },
      newPassword: SAYS_NOTHING,
      unexplained: false,
    };
  }

  const newPassword = messageOf(passwordRejection(error, 'new_password'));
  const currentPassword = messageOf(validationRejection(error, 'current_password'));

  return {
    currentPassword,
    newPassword,
    unexplained: !currentPassword.shown && !newPassword.shown,
  };
}

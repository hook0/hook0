import { problemCarriedBy, validationErrorsFor, type ProblemLike } from '@/utils/problem';

/**
 * The API errors that mean "this password was refused", as opposed to "the
 * request failed". Two of the six rules — the blocklist and the length ceiling
 * — can only be checked on the server, so a rejection arriving as an API error
 * is a normal part of setting a password, not an incident: it belongs next to
 * the field the user must fix, not in a toast that leaves the form looking
 * fine.
 *
 * Kept in step with `Rejection::into_problem` (api/src/password.rs) by
 * password-policy-vectors.json, which both test suites read.
 */
export const PASSWORD_REJECTION_PROBLEM_IDS = [
  'PasswordTooShort',
  'PasswordTooLong',
  'PasswordSimilarToEmail',
  'PasswordSimilarToName',
  'PasswordTooCommon',
  'PasswordNotDiverseEnough',
] as const;

const REJECTION_IDS: ReadonlySet<string> = new Set(PASSWORD_REJECTION_PROBLEM_IDS);

export type PasswordRejection =
  { readonly refused: false } | { readonly refused: true; readonly reason: string };

const NOT_A_REJECTION: PasswordRejection = { refused: false };

/**
 * The policy's verdict on the password a request carried. It names the rule
 * that refused it and no field at all — the API has no reason to, since only
 * one password per request is ever run through the policy — so the caller is
 * the one that knows which field it belongs under.
 */
function verdictFor(problem: ProblemLike): PasswordRejection {
  if (!REJECTION_IDS.has(problem.id)) {
    return NOT_A_REJECTION;
  }
  return { refused: true, reason: problem.detail };
}

/**
 * The other way a password comes back refused: not as one of the policy's own
 * errors, but as a generic `Validation` 422 — a control character pasted out
 * of a password manager, say. The API already names the offending field in
 * `validation`, and nothing in the app read it, so those refusals arrived as a
 * toast above a field that still looked accepted.
 *
 * Only errors carrying a human-readable message are surfaced; the built-in
 * validators leave it null and their raw code would mean nothing to a user.
 */
function validationMessageFor(problem: ProblemLike, field: string): PasswordRejection {
  const errors = validationErrorsFor(problem, field);
  if (!errors.named) {
    return NOT_A_REJECTION;
  }
  for (const entry of errors.entries) {
    if (entry === null || typeof entry !== 'object') {
      continue;
    }
    const message: unknown = (entry as Record<string, unknown>).message;
    if (typeof message === 'string' && message !== '') {
      return { refused: true, reason: message };
    }
  }
  return NOT_A_REJECTION;
}

/**
 * Read a failed password request, for the field the policy ran on.
 *
 * `field` scopes the `Validation` 422 half of the answer — it is the name the
 * API knows the password by on this endpoint, `password` on registration,
 * `new_password` on reset and change — so a validation error about somebody's
 * first name is never blamed on it.
 *
 * It does not scope the policy half, which carries no field to scope by. On an
 * endpoint sending two passwords that makes this the wrong thing to ask about
 * the one that only proves who is asking: it would answer with the *other*
 * field's reason. `validationRejection` is that question.
 */
export function passwordRejection(error: unknown, field: string): PasswordRejection {
  const carrier = problemCarriedBy(error);
  if (!carrier.carries) {
    return NOT_A_REJECTION;
  }
  const rejection = verdictFor(carrier.problem);
  if (rejection.refused) {
    return rejection;
  }
  return validationMessageFor(carrier.problem, field);
}

/**
 * Read a failed password request for a field the policy never runs on — the
 * current password of a change, which is compared rather than checked against
 * the rules. Only refusals the API pinned on that exact field come back, so
 * the new password's reason can never be shown under it.
 */
export function validationRejection(error: unknown, field: string): PasswordRejection {
  const carrier = problemCarriedBy(error);
  if (!carrier.carries) {
    return NOT_A_REJECTION;
  }
  return validationMessageFor(carrier.problem, field);
}

/**
 * How `POST /auth/password` refuses to change the password. The API answers
 * `Forbidden` — the same problem it raises for a caller with no right to make
 * the request, on purpose: a distinct error would tell an attacker holding a
 * stolen session which half of the request it got wrong.
 *
 * So this is *a* refusal, not proof of a mistyped password. `authorize_only_user`
 * (api/src/iam.rs) raises the same `Forbidden` before the password is ever
 * checked, for a service or master token, or a user token the authorizer
 * attenuated. The dashboard sends none of those — it holds a plain user access
 * token — which is why the verdict is shown on the field rather than in a
 * toast; but the message that goes there says the password was not accepted,
 * never that it was wrong, because that is the part this cannot know.
 *
 * An expired session is not in the ambiguity: the middleware checks the
 * token row's `expired_at` and answers `AuthInvalidBiscuit`
 * (api/src/middleware_biscuit.rs), so a stale tab never blames the password.
 */
const CURRENT_PASSWORD_REFUSAL_ID = 'Forbidden';

export function isCurrentPasswordRefused(error: unknown): boolean {
  const carrier = problemCarriedBy(error);
  return carrier.carries && carrier.problem.id === CURRENT_PASSWORD_REFUSAL_ID;
}

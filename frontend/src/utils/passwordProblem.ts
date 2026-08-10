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

interface ProblemLike {
  readonly id: string;
  readonly detail: string;
}

function isProblemLike(value: unknown): value is ProblemLike {
  if (value === null || typeof value !== 'object') {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return typeof candidate.id === 'string' && typeof candidate.detail === 'string';
}

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
  const validation: unknown = (problem as unknown as Record<string, unknown>).validation;
  if (validation === null || typeof validation !== 'object') {
    return NOT_A_REJECTION;
  }
  const entries: unknown = (validation as Record<string, unknown>)[field];
  if (!Array.isArray(entries)) {
    return NOT_A_REJECTION;
  }
  for (const entry of entries as unknown[]) {
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

function verdict(problem: ProblemLike, field: string): PasswordRejection {
  const rejection = verdictFor(problem);
  if (rejection.refused) {
    return rejection;
  }
  return validationMessageFor(problem, field);
}

/**
 * Read a failed password request. Accepts either shape the app rejects with:
 * a Problem already unwrapped by `unwrapResponse`, or the raw Axios error that
 * still carries it in `response.data`. Anything else is a failed request, and
 * says nothing about the password.
 *
 * `field` is the name the API knows the password by on this endpoint —
 * `password` on registration, `new_password` on reset and change — so a
 * validation error about somebody's first name is never blamed on it.
 */
export function passwordRejection(error: unknown, field: string): PasswordRejection {
  if (isProblemLike(error)) {
    return verdict(error, field);
  }
  if (error === null || typeof error !== 'object') {
    return NOT_A_REJECTION;
  }
  const response: unknown = (error as Record<string, unknown>).response;
  if (response === null || typeof response !== 'object') {
    return NOT_A_REJECTION;
  }
  const data: unknown = (response as Record<string, unknown>).data;
  if (isProblemLike(data)) {
    return verdict(data, field);
  }
  return NOT_A_REJECTION;
}

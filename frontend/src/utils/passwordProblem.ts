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
 * Read a failed password request. Accepts either shape the app rejects with:
 * a Problem already unwrapped by `unwrapResponse`, or the raw Axios error that
 * still carries it in `response.data`. Anything else is a failed request, and
 * says nothing about the password.
 */
export function passwordRejection(error: unknown): PasswordRejection {
  if (isProblemLike(error)) {
    return verdictFor(error);
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
    return verdictFor(data);
  }
  return NOT_A_REJECTION;
}

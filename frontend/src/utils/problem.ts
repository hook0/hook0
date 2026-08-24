/**
 * The RFC 7807 body every Hook0 error answers with, read from whichever shape
 * a rejected promise carries it in: a Problem already unwrapped by
 * `unwrapResponse`, or the raw Axios error that still holds it in
 * `response.data`.
 *
 * Both shapes are read in one place because a reader that handles one and not
 * the other looks right in review and goes quiet exactly when a caller stops
 * going through `unwrapResponse`.
 */
export interface ProblemLike {
  readonly id: string;
  readonly detail: string;
}

/**
 * A rejected request either carries a problem body or it does not. Modelled as
 * a sum rather than a possibly-absent object so no caller can forget the
 * second case.
 */
export type ProblemCarrier =
  { readonly carries: false } | { readonly carries: true; readonly problem: ProblemLike };

const CARRIES_NO_PROBLEM: ProblemCarrier = { carries: false };

function isProblemLike(value: unknown): value is ProblemLike {
  if (value === null || typeof value !== 'object') {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return typeof candidate.id === 'string' && typeof candidate.detail === 'string';
}

export function problemCarriedBy(error: unknown): ProblemCarrier {
  if (isProblemLike(error)) {
    return { carries: true, problem: error };
  }
  if (error === null || typeof error !== 'object') {
    return CARRIES_NO_PROBLEM;
  }
  const response: unknown = (error as Record<string, unknown>).response;
  if (response === null || typeof response !== 'object') {
    return CARRIES_NO_PROBLEM;
  }
  const data: unknown = (response as Record<string, unknown>).data;
  if (isProblemLike(data)) {
    return { carries: true, problem: data };
  }
  return CARRIES_NO_PROBLEM;
}

/**
 * What a `Validation` 422 says about one field. The API answers with a map
 * keyed on the request struct's field names, so a refusal about somebody's
 * first name is never mistaken for one about their password.
 *
 * Absent and present-but-empty are both worth telling apart from "not an
 * object at all": the errors a built-in validator raises carry no readable
 * message, and a caller may still want to know the field was named.
 */
export type FieldErrors =
  { readonly named: false } | { readonly named: true; readonly entries: readonly unknown[] };

const FIELD_NOT_NAMED: FieldErrors = { named: false };

export function validationErrorsFor(problem: ProblemLike, field: string): FieldErrors {
  const validation: unknown = (problem as unknown as Record<string, unknown>).validation;
  if (validation === null || typeof validation !== 'object') {
    return FIELD_NOT_NAMED;
  }
  const entries: unknown = (validation as Record<string, unknown>)[field];
  if (!Array.isArray(entries)) {
    return FIELD_NOT_NAMED;
  }
  return { named: true, entries: entries as readonly unknown[] };
}

/**
 * Whether a request was refused over the shape of a field the caller sent,
 * rather than over anything the account behind it does or does not have.
 *
 * The distinction is what makes this safe to surface on a page that must not
 * say whether an address is registered: the request struct's own validators
 * run before any lookup, so their verdict is the same for an address that
 * exists and one that does not.
 */
export function isFieldRefused(error: unknown, field: string): boolean {
  const carrier = problemCarriedBy(error);
  return carrier.carries && validationErrorsFor(carrier.problem, field).named;
}

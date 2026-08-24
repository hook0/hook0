import { problemCarriedBy } from '@/utils/problem';

/**
 * The problems that mean the request never got as far as a decision about the
 * address it carried.
 *
 * Two are synthesised client-side by `handleError` (frontend/src/http.ts) when
 * the server never got to state one: a request that timed out, and one whose
 * reply carried no RFC 7807 body — a transport failure, or a 5xx served by
 * something in front of the API.
 *
 * The other two the API does pronounce, and they are read the same way on
 * purpose: an internal error or an unavailable service on this endpoint comes
 * from the database call that claims the send, which runs before anything about
 * the address is known and fails identically whether or not an account exists.
 */
const UNFULFILLED_PROBLEM_IDS = [
  'TimeoutExceeded',
  'unknown',
  'InternalServerError',
  'ServiceUnavailable',
] as const;

/**
 * Whether the request failed before it could decide anything about the address.
 *
 * This matters where a failure is otherwise reported as success on purpose. A
 * page that hides what the API said about an address must still not claim a
 * mail was sent when none was ever minted: silence protects the account that
 * may exist, it does not protect a user staring at a promise that will never
 * arrive. Telling these apart leaks nothing — none of them is a verdict about
 * the address.
 */
export function isUnfulfilled(error: unknown): boolean {
  const carrier = problemCarriedBy(error);
  return carrier.carries && UNFULFILLED_PROBLEM_IDS.some((id) => id === carrier.problem.id);
}

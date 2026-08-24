import { problemCarriedBy } from '@/utils/problem';

/**
 * The problem every Hook0 rate limiter answers with. The limiters sit outside
 * the handlers, so they build the RFC 7807 body themselves
 * (`rate_limited_response`, api/src/rate_limiting.rs) from the one variant
 * named in api/src/problems.rs — 429, with a `Retry-After` header.
 */
const RATE_LIMITED_PROBLEM_ID = 'RateLimited';

/**
 * Whether a request was refused for pacing rather than for anything it carried.
 * True of the request, never of the account behind it: the limiters key on the
 * caller's IP, so this verdict is the same for an address that exists and one
 * that does not.
 */
export function isRateLimited(error: unknown): boolean {
  const carrier = problemCarriedBy(error);
  return carrier.carries && carrier.problem.id === RATE_LIMITED_PROBLEM_ID;
}

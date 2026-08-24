import fc from 'fast-check';

import { isRateLimited } from './rateLimited';

describe('isRateLimited', () => {
  // The body a Hook0 limiter actually returns, copied from the variant in
  // api/src/problems.rs rather than paraphrased.
  const refusal = {
    id: 'RateLimited',
    status: 429,
    title: 'Too many requests',
    detail:
      'Requests are coming in faster than this Hook0 instance accepts them, so this one was not processed.',
  };

  it('recognises the problem every limiter answers with', () => {
    expect(isRateLimited(refusal)).toBe(true);
  });

  // The other shape a rejected promise carries the same body in. Every page
  // reading this goes through `unwrapResponse`, which normalises the two — but
  // a reader that knows only one shape reports "not a rate limit" the day a
  // caller catches ahead of it, and the page then swallows a 429 as if it were
  // any other failure.
  it('reads a raw Axios error carrying the Problem in its response', () => {
    expect(isRateLimited({ isAxiosError: true, response: { data: refusal } })).toBe(true);
  });

  // The distinction the forgot-password page is built on. `AuthEmailExpired`
  // is what that endpoint used to answer for an address it did not know, and
  // showing it is the enumeration oracle itself.
  it('leaves the errors that must stay silent alone', () => {
    expect(
      isRateLimited({
        id: 'AuthEmailExpired',
        status: 401,
        title: 'Expired email',
        detail: 'This email is unknown or its link expired.',
      })
    ).toBe(false);
    expect(
      isRateLimited({
        id: 'InternalServerError',
        status: 500,
        title: 'Internal server error',
        detail: 'Something went wrong.',
      })
    ).toBe(false);
  });

  it('survives anything a rejected promise can carry', () => {
    fc.assert(
      fc.property(fc.anything(), (value) => typeof isRateLimited(value) === 'boolean'),
      { numRuns: 500, seed: 20260823 }
    );
  });

  // The whole contract: the verdict tracks the problem id and nothing else.
  // Keying on the 429 status instead would catch limiters this page must stay
  // silent about; being looser than an exact id match would leak the oracle
  // through whichever error happened to slip past.
  it('claims a rate limit for exactly one problem id, whatever else the Problem says', () => {
    fc.assert(
      fc.property(
        fc.string(),
        fc.integer({ min: 100, max: 599 }),
        fc.string(),
        fc.string(),
        fc.boolean(),
        (id, status, title, detail, wrapped) => {
          const problem = { id, status, title, detail };
          const carrier = wrapped ? { isAxiosError: true, response: { data: problem } } : problem;
          return isRateLimited(carrier) === (id === 'RateLimited');
        }
      ),
      { numRuns: 500, seed: 20260823 }
    );
  });

  // A 429 that is not a Hook0 problem body — a proxy or CDN refusing ahead of
  // the API — carries no `id`, and guessing from the status alone would put a
  // "wait and retry" message under an outage the user cannot wait out.
  it('does not read a rate limit out of a body that names no problem', () => {
    expect(isRateLimited({ status: 429 })).toBe(false);
    expect(isRateLimited({ id: 429 })).toBe(false);
    expect(isRateLimited({ isAxiosError: true, response: { data: '<html>429</html>' } })).toBe(
      false
    );
  });
});

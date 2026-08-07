import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import fc from 'fast-check';

import { PASSWORD_REJECTION_PROBLEM_IDS, passwordRejection } from './passwordProblem';

// The same file drives the Rust suite (api/src/password.rs), which checks the
// list against `Rejection::into_problem`. Read rather than imported so the
// bundler never sees it: it is a test fixture, not shipped code.
const sharedRejectionIds: string[] = (
  JSON.parse(readFileSync(join(__dirname, '../../../password-policy-vectors.json'), 'utf8')) as {
    rejectionProblemIds: string[];
  }
).rejectionProblemIds;

describe('passwordRejection', () => {
  // The contract with the API. A reason added on the server without its id
  // here would reach the user as a toast with no field to fix.
  it('covers exactly the rejection reasons shared with the API', () => {
    expect([...PASSWORD_REJECTION_PROBLEM_IDS].sort()).toEqual([...sharedRejectionIds].sort());
  });

  it('reads a Problem already unwrapped by the HTTP layer', () => {
    expect(
      passwordRejection({
        id: 'PasswordTooCommon',
        status: 400,
        title: 'Password too common',
        detail: 'This password appears in lists of commonly used passwords.',
      })
    ).toEqual({
      refused: true,
      reason: 'This password appears in lists of commonly used passwords.',
    });
  });

  it('reads a raw Axios error carrying the Problem in its response', () => {
    expect(
      passwordRejection({
        isAxiosError: true,
        response: {
          data: {
            id: 'PasswordSimilarToEmail',
            status: 400,
            title: 'Password similar to email',
            detail: 'Your password must not be built from your email address.',
          },
        },
      })
    ).toEqual({
      refused: true,
      reason: 'Your password must not be built from your email address.',
    });
  });

  // Everything else is a failed request, and belongs in the toast the caller
  // already shows — not pinned to the password field as if the user had typed
  // something wrong.
  it('leaves unrelated errors alone', () => {
    expect(
      passwordRejection({
        id: 'AuthFailedRefreshToken',
        status: 401,
        title: 'Refresh failed',
        detail: 'Session expired.',
      })
    ).toEqual({ refused: false });
  });

  it('survives anything a rejected promise can carry', () => {
    fc.assert(
      fc.property(fc.anything(), (value) => {
        const verdict = passwordRejection(value);
        // A rejection reason is always a string the form can display.
        return verdict.refused === false || typeof verdict.reason === 'string';
      }),
      { numRuns: 500, seed: 20260806 }
    );
  });

  it('never claims a refusal for an id the API does not use for passwords', () => {
    fc.assert(
      fc.property(
        fc.string(),
        fc.string(),
        (id, detail) =>
          passwordRejection({ id, status: 400, title: '', detail }).refused ===
          sharedRejectionIds.includes(id)
      ),
      { numRuns: 500, seed: 20260806 }
    );
  });
});

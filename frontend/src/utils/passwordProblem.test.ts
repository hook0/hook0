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
      passwordRejection(
        {
          id: 'PasswordTooCommon',
          status: 400,
          title: 'Password too common',
          detail: 'This password appears in lists of commonly used passwords.',
        },
        'password'
      )
    ).toEqual({
      refused: true,
      reason: 'This password appears in lists of commonly used passwords.',
    });
  });

  it('reads a raw Axios error carrying the Problem in its response', () => {
    expect(
      passwordRejection(
        {
          isAxiosError: true,
          response: {
            data: {
              id: 'PasswordSimilarToEmail',
              status: 400,
              title: 'Password similar to email',
              detail: 'Your password must not be built from your email address.',
            },
          },
        },
        'password'
      )
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
      passwordRejection(
        {
          id: 'AuthFailedRefreshToken',
          status: 401,
          title: 'Refresh failed',
          detail: 'Session expired.',
        },
        'password'
      )
    ).toEqual({ refused: false });
  });

  describe('generic validation errors', () => {
    // A control character pasted out of a password manager comes back as a
    // plain 422, not as one of the policy's own errors. The API names the
    // offending field; without reading it the user gets "provided input is
    // malformed" over a field that still looks accepted.
    const controlCharacterRefusal = {
      id: 'Validation',
      status: 422,
      title: 'Provided input is malformed',
      detail: 'password: Password must not contain control characters',
      validation: {
        password: [
          {
            code: 'secret-characters',
            message: 'Password must not contain control characters',
            params: {},
          },
        ],
      },
    };

    it('surfaces the message the API attached to the password field', () => {
      expect(passwordRejection(controlCharacterRefusal, 'password')).toEqual({
        refused: true,
        reason: 'Password must not contain control characters',
      });
    });

    // The same 422 on registration can just as easily be about the first name.
    // Blaming the password field for it would send the user to fix the wrong
    // input.
    it('does not blame the password for another field', () => {
      expect(
        passwordRejection(
          {
            id: 'Validation',
            status: 422,
            title: 'Provided input is malformed',
            detail: 'first_name: too long',
            validation: { first_name: [{ code: 'length', message: 'Too long', params: {} }] },
          },
          'password'
        )
      ).toEqual({ refused: false });
    });

    // The built-in validators leave the message null, and their raw code
    // ("length", "email") would mean nothing to a user.
    it('stays silent when the error carries no readable message', () => {
      expect(
        passwordRejection(
          {
            id: 'Validation',
            status: 422,
            title: 'Provided input is malformed',
            detail: 'password: invalid',
            validation: { password: [{ code: 'length', message: null, params: {} }] },
          },
          'password'
        )
      ).toEqual({ refused: false });
    });

    it('looks under the name the endpoint uses for its password field', () => {
      const onReset = {
        ...controlCharacterRefusal,
        validation: { new_password: controlCharacterRefusal.validation.password },
      };
      expect(passwordRejection(onReset, 'new_password').refused).toBe(true);
      expect(passwordRejection(onReset, 'password').refused).toBe(false);
    });
  });

  it('survives anything a rejected promise can carry', () => {
    fc.assert(
      fc.property(fc.anything(), (value) => {
        const verdict = passwordRejection(value, 'password');
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
          passwordRejection({ id, status: 400, title: '', detail }, 'password').refused ===
          sharedRejectionIds.includes(id)
      ),
      { numRuns: 500, seed: 20260806 }
    );
  });
});

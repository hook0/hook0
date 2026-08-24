import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import fc from 'fast-check';

import {
  PASSWORD_REJECTION_PROBLEM_IDS,
  isCurrentPasswordRefused,
  passwordRejection,
  validationRejection,
} from './passwordProblem';

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

    // The map is JSON the API built; a reader that assumes every entry is an
    // object crashes the whole catch block on the first one that is not, and
    // takes the toast down with it.
    it('walks past entries that carry nothing readable and keeps looking', () => {
      expect(
        passwordRejection(
          {
            id: 'Validation',
            status: 422,
            title: 'Provided input is malformed',
            detail: 'password: invalid',
            validation: {
              password: [null, 'malformed', { code: 'secret-characters', message: 'Not this' }],
            },
          },
          'password'
        )
      ).toEqual({ refused: true, reason: 'Not this' });
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

describe('isCurrentPasswordRefused', () => {
  const refusal = {
    id: 'Forbidden',
    status: 403,
    title: 'Insufficient rights',
    detail: "You don't have the right to access or edit this resource.",
  };

  it('reads a Problem already unwrapped by the HTTP layer', () => {
    expect(isCurrentPasswordRefused(refusal)).toBe(true);
  });

  it('reads a raw Axios error carrying the Problem in its response', () => {
    expect(isCurrentPasswordRefused({ isAxiosError: true, response: { data: refusal } })).toBe(
      true
    );
  });

  // The session errors the biscuit middleware raises share the endpoint but
  // not the meaning: telling the user to retype a password they got right
  // would send them chasing a session problem they cannot see.
  it('leaves an expired session alone', () => {
    expect(
      isCurrentPasswordRefused({
        id: 'AuthInvalidBiscuit',
        status: 401,
        title: 'Invalid token',
        detail: 'Your session is no longer valid.',
      })
    ).toBe(false);
  });

  it('survives anything a rejected promise can carry', () => {
    fc.assert(
      fc.property(fc.anything(), (value) => typeof isCurrentPasswordRefused(value) === 'boolean'),
      { numRuns: 500, seed: 20260823 }
    );
  });

  // A body missing the field entirely fails serde before any validator runs,
  // so the API answers `JsonPayload` with no `validation` map at all. The form
  // cannot produce this — the field is required and always sent — but reading
  // a key off an absent map is how a diagnosis becomes a crash.
  it('does not mistake a malformed payload for a refusal', () => {
    const malformed = {
      id: 'JsonPayload',
      status: 400,
      title: 'Provided JSON payload could not be processed',
      detail: 'missing field `current_password`',
    };
    expect(isCurrentPasswordRefused(malformed)).toBe(false);
    expect(passwordRejection(malformed, 'current_password')).toEqual({ refused: false });
  });

  // The whole contract in one line: the verdict tracks the problem id and
  // nothing else. A refusal invented for any other failure — a timeout, a 500,
  // a rate limit — would pin a wrong-password message on a field the user
  // filled in correctly.
  it('claims a refusal for exactly one problem id, whatever else the Problem says', () => {
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
          return isCurrentPasswordRefused(carrier) === (id === 'Forbidden');
        }
      ),
      { numRuns: 500, seed: 20260823 }
    );
  });
});

describe('validationRejection', () => {
  // What separates it from `passwordRejection`: the policy's verdict carries
  // no field, so a reader that let it through would answer with the *other*
  // password's reason on a request that sends two.
  it('never answers with a verdict the policy did not pin on a field', () => {
    fc.assert(
      fc.property(fc.constantFrom(...sharedRejectionIds), fc.string(), (id, detail) =>
        [
          validationRejection({ id, status: 400, title: '', detail }, 'current_password').refused,
          validationRejection({ id, status: 400, title: '', detail }, 'new_password').refused,
        ].every((refused) => refused === false)
      ),
      { numRuns: 500, seed: 20260824 }
    );
  });

  it('answers with the message the API pinned on the field it was asked about', () => {
    const malformed = {
      id: 'Validation',
      status: 422,
      title: 'Provided input is malformed',
      detail: 'current_password: Password must not contain control characters',
      validation: {
        current_password: [
          {
            code: 'secret-characters',
            message: 'Password must not contain control characters',
            params: {},
          },
        ],
      },
    };

    expect(validationRejection(malformed, 'current_password')).toEqual({
      refused: true,
      reason: 'Password must not contain control characters',
    });
    expect(validationRejection(malformed, 'new_password')).toEqual({ refused: false });
  });

  it('reads a raw Axios error carrying the Problem in its response', () => {
    expect(
      validationRejection(
        {
          isAxiosError: true,
          response: {
            data: {
              id: 'Validation',
              status: 422,
              title: 'Provided input is malformed',
              detail: 'new_password: nope',
              validation: { new_password: [{ code: 'secret-characters', message: 'Nope' }] },
            },
          },
        },
        'new_password'
      )
    ).toEqual({ refused: true, reason: 'Nope' });
  });

  it('survives anything a rejected promise can carry', () => {
    fc.assert(
      fc.property(fc.anything(), (value) => {
        const verdict = validationRejection(value, 'current_password');
        return verdict.refused === false || typeof verdict.reason === 'string';
      }),
      { numRuns: 500, seed: 20260824 }
    );
  });
});

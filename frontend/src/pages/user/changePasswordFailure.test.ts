import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import fc from 'fast-check';

import { changePasswordFailure } from './changePasswordFailure';

// The same file drives the Rust suite (api/src/password.rs). Read rather than
// imported so the bundler never sees it: it is a test fixture, not shipped code.
const policyRejectionIds: string[] = (
  JSON.parse(readFileSync(join(__dirname, '../../../../password-policy-vectors.json'), 'utf8')) as {
    rejectionProblemIds: string[];
  }
).rejectionProblemIds;

const CURRENT_PASSWORD_REFUSED = 'Your current password was not accepted.';

describe('changePasswordFailure', () => {
  // The regression this exists for. The blocklist does not ship to the
  // browser, so `PasswordTooCommon` can only arrive as an API error — and it
  // is about the password being installed, never about the one that proved
  // who was asking. Reported on "Current password" it sends someone away to
  // retype a password that was right, with a message about a different field.
  it('puts a policy refusal on the new password and nowhere else', () => {
    const failure = changePasswordFailure(
      {
        id: 'PasswordTooCommon',
        status: 400,
        title: 'Password too common',
        detail: 'This password is too frequently used.',
      },
      CURRENT_PASSWORD_REFUSED
    );

    expect(failure.newPassword).toEqual({
      shown: true,
      message: 'This password is too frequently used.',
    });
    expect(failure.currentPassword).toEqual({ shown: false });
    expect(failure.unexplained).toBe(false);
  });

  it('reports a refused current password under the field that holds it', () => {
    const failure = changePasswordFailure(
      {
        id: 'Forbidden',
        status: 403,
        title: 'Insufficient rights',
        detail: "You don't have the right to access or edit this resource.",
      },
      CURRENT_PASSWORD_REFUSED
    );

    expect(failure.currentPassword).toEqual({ shown: true, message: CURRENT_PASSWORD_REFUSED });
    // And not the API's own wording, which is about rights rather than about a
    // password and would only puzzle someone who can just retype one.
    expect(failure.newPassword).toEqual({ shown: false });
    expect(failure.unexplained).toBe(false);
  });

  // The one refusal that really is about the current password's own content:
  // a control character pasted out of a password manager trips the request
  // struct's validator, which names the field it was pasted into.
  it('follows the field a validation refusal names', () => {
    const malformed = (field: string) => ({
      id: 'Validation',
      status: 422,
      title: 'Provided input is malformed',
      detail: `${field}: Password must not contain control characters`,
      validation: {
        [field]: [
          { code: 'secret-characters', message: 'Password must not contain control characters' },
        ],
      },
    });

    const onCurrent = changePasswordFailure(
      malformed('current_password'),
      CURRENT_PASSWORD_REFUSED
    );
    expect(onCurrent.currentPassword).toEqual({
      shown: true,
      message: 'Password must not contain control characters',
    });
    expect(onCurrent.newPassword).toEqual({ shown: false });

    const onNew = changePasswordFailure(malformed('new_password'), CURRENT_PASSWORD_REFUSED);
    expect(onNew.newPassword).toEqual({
      shown: true,
      message: 'Password must not contain control characters',
    });
    expect(onNew.currentPassword).toEqual({ shown: false });
  });

  it('reports both fields when the refusal names both', () => {
    const failure = changePasswordFailure(
      {
        id: 'Validation',
        status: 422,
        title: 'Provided input is malformed',
        detail: 'current_password: bad, new_password: bad',
        validation: {
          current_password: [{ code: 'secret-characters', message: 'The old one is malformed' }],
          new_password: [{ code: 'secret-characters', message: 'The new one is malformed' }],
        },
      },
      CURRENT_PASSWORD_REFUSED
    );

    expect(failure.currentPassword).toEqual({ shown: true, message: 'The old one is malformed' });
    expect(failure.newPassword).toEqual({ shown: true, message: 'The new one is malformed' });
    expect(failure.unexplained).toBe(false);
  });

  // Everything the form cannot explain has to be reported somewhere else. A
  // failure silently dropped leaves a form that looks like it was never
  // submitted.
  it('leaves a failure the form cannot explain to the caller', () => {
    const failure = changePasswordFailure(
      {
        id: 'ServiceUnavailable',
        status: 503,
        title: 'Something wrong happened',
        detail: 'Hook0 is busy, please retry.',
      },
      CURRENT_PASSWORD_REFUSED
    );

    expect(failure.unexplained).toBe(true);
    expect(failure.currentPassword).toEqual({ shown: false });
    expect(failure.newPassword).toEqual({ shown: false });
  });

  // The attribution rule, over the whole shared list rather than the one id
  // that happened to be reachable by hand: no reason the policy raises may
  // ever land on the current password.
  it('never blames the current password for a rule the policy applies to the new one', () => {
    fc.assert(
      fc.property(fc.constantFrom(...policyRejectionIds), fc.string(), (id, detail) => {
        const failure = changePasswordFailure(
          { id, status: 400, title: '', detail },
          CURRENT_PASSWORD_REFUSED
        );
        return (
          failure.currentPassword.shown === false &&
          failure.newPassword.shown === true &&
          failure.newPassword.message === detail
        );
      }),
      { numRuns: 500, seed: 20260824 }
    );
  });

  it('survives anything a rejected promise can carry', () => {
    fc.assert(
      fc.property(fc.anything(), (value) => {
        const failure = changePasswordFailure(value, CURRENT_PASSWORD_REFUSED);
        return (
          failure.unexplained === (!failure.currentPassword.shown && !failure.newPassword.shown)
        );
      }),
      { numRuns: 500, seed: 20260824 }
    );
  });
});

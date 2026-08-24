import { createPasswordChangeSchema } from './passwordChange.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import type { UserIdentity } from '@/utils/passwordPolicy';

// The same account the policy suite uses, so the weak vectors below are ones
// `checkPassword` is already proven to refuse rather than ones guessed here.
const identity: UserIdentity = {
  email: 'jordanrivera801@example.com',
  firstName: 'Jordan',
  lastName: 'Rivera',
};

const MINIMUM_LENGTH = 12;

const schema = createPasswordChangeSchema(identity, MINIMUM_LENGTH);

function issuePaths(values: {
  current_password: string;
  new_password: string;
  confirm_new_password: string;
}): string[] {
  const result = schema.safeParse(values);
  if (result.success) {
    return [];
  }
  return result.error.issues.map((issue) => issue.path.join('.'));
}

describe('createPasswordChangeSchema', () => {
  it('accepts a strong new password confirmed, with the current one presented', () => {
    expect(
      issuePaths({
        current_password: 'the-one-in-use',
        new_password: 'correct-horse-battery',
        confirm_new_password: 'correct-horse-battery',
      })
    ).toEqual([]);
  });

  it('requires the current password', () => {
    expect(
      issuePaths({
        current_password: '',
        new_password: 'correct-horse-battery',
        confirm_new_password: 'correct-horse-battery',
      })
    ).toEqual(['current_password']);
  });

  // The regression this guards against: adding a required field above these two
  // must not swallow the diagnosis they already gave. Someone who types a
  // mismatched or weak new password has to be told so while the current-password
  // box is still empty — otherwise the form goes quiet and leaves them with a
  // disabled button and no reason for it.
  describe('while the current password is still empty', () => {
    it('still reports mismatched passwords', () => {
      expect(
        issuePaths({
          current_password: '',
          new_password: 'correct-horse-battery',
          confirm_new_password: 'a-different-one-entirely',
        })
      ).toEqual(['current_password', 'confirm_new_password']);
    });

    it('still reports a password built from the account it protects', () => {
      expect(
        issuePaths({
          current_password: '',
          new_password: 'xx-jordanrivera801-xx',
          confirm_new_password: 'xx-jordanrivera801-xx',
        })
      ).toEqual(['current_password', 'new_password']);
    });

    it('still reports a new password under the instance floor', () => {
      expect(
        issuePaths({
          current_password: '',
          new_password: 'short',
          confirm_new_password: 'short',
        })
      ).toEqual(['current_password', 'new_password']);
    });
  });

  // The other face: filling in the current password must not silence anything
  // either, and the weakness rule keeps its own path so the error lands on the
  // field the user has to change.
  describe('once the current password is filled in', () => {
    it('reports mismatched passwords on the confirmation field', () => {
      expect(
        issuePaths({
          current_password: 'the-one-in-use',
          new_password: 'correct-horse-battery',
          confirm_new_password: 'a-different-one-entirely',
        })
      ).toEqual(['confirm_new_password']);
    });

    it('reports a password built from the account on the new-password field', () => {
      expect(
        issuePaths({
          current_password: 'the-one-in-use',
          new_password: 'xx-jordanrivera801-xx',
          confirm_new_password: 'xx-jordanrivera801-xx',
        })
      ).toEqual(['new_password']);
    });
  });
});

// The schema is only half the path. What the field binding reads is whatever
// the vee-validate adapter hands back, so the guarantee above is worth nothing
// unless it survives that translation — these assert the same property one
// layer closer to what the user sees.
describe('the errors the form binding receives', () => {
  const typed = toTypedSchema(createPasswordChangeSchema(identity, MINIMUM_LENGTH));

  function errorPaths(values: {
    current_password: string;
    new_password: string;
    confirm_new_password: string;
  }): Promise<string[]> {
    return typed
      .parse(values)
      .then((result) => (result.errors as { path?: string }[]).map((error) => String(error.path)));
  }

  it('carries the mismatch to the confirmation field while the current one is empty', () => {
    return errorPaths({
      current_password: '',
      new_password: 'correct-horse-battery',
      confirm_new_password: 'a-different-one-entirely',
    }).then((paths) => {
      expect(paths).toContain('confirm_new_password');
      expect(paths).toContain('current_password');
    });
  });

  it('carries the weakness to the new-password field while the current one is empty', () => {
    return errorPaths({
      current_password: '',
      new_password: 'xx-jordanrivera801-xx',
      confirm_new_password: 'xx-jordanrivera801-xx',
    }).then((paths) => {
      expect(paths).toContain('new_password');
      expect(paths).toContain('current_password');
    });
  });
});

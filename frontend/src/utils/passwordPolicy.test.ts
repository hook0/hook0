import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import fc from 'fast-check';

import { checkPassword, foldIdentity, type UserIdentity } from './passwordPolicy';

type SharedVector = {
  why: string;
  password: string;
  email: string;
  firstName: string;
  lastName: string;
  verdict: 'acceptable' | 'similarToEmail' | 'similarToName';
};

// The same file drives the Rust suite (api/src/password.rs). Read rather than
// imported so the bundler never sees it: it is a test fixture, not shipped code.
const sharedVectors: SharedVector[] = (
  JSON.parse(readFileSync(join(__dirname, '../../../password-policy-vectors.json'), 'utf8')) as {
    vectors: SharedVector[];
  }
).vectors;

const identity: UserIdentity = {
  email: 'jordanrivera801@example.com',
  firstName: 'Jordan',
  lastName: 'Rivera',
};

describe('checkPassword', () => {
  // The reported vulnerability: an account whose password is its own email
  // address. This is the case the whole module exists for.
  it('refuses a password equal to the email address', () => {
    expect(checkPassword(identity.email, identity)).toEqual({
      acceptable: false,
      weakness: 'similarToEmail',
    });
  });

  it('refuses the email address whatever its case', () => {
    expect(checkPassword('JordanRivera801@Example.com', identity)).toEqual({
      acceptable: false,
      weakness: 'similarToEmail',
    });
  });

  it('refuses the local part of the email address', () => {
    expect(checkPassword('jordanrivera801', identity)).toEqual({
      acceptable: false,
      weakness: 'similarToEmail',
    });
  });

  it('refuses a password built around the email address', () => {
    expect(checkPassword('xx-jordanrivera801-xx', identity)).toEqual({
      acceptable: false,
      weakness: 'similarToEmail',
    });
  });

  it('refuses a password built around the user name', () => {
    expect(checkPassword('rivera-is-here', identity)).toEqual({
      acceptable: false,
      weakness: 'similarToName',
    });
  });

  it('sees through lookalike substitutions', () => {
    expect(checkPassword('j0rd4nrivera801@example.com', identity)).toEqual({
      acceptable: false,
      weakness: 'similarToEmail',
    });
  });

  it('refuses a password that is mostly the local part', () => {
    const dotted: UserIdentity = {
      email: 'john.doe@example.com',
      firstName: '',
      lastName: '',
    };
    expect(checkPassword('john.doe2026', dotted)).toEqual({
      acceptable: false,
      weakness: 'similarToEmail',
    });
  });

  it('refuses each half of a dotted local part', () => {
    const dotted: UserIdentity = {
      email: 'john.doe@example.com',
      firstName: '',
      lastName: '',
    };
    expect(checkPassword('john', dotted)).toEqual({
      acceptable: false,
      weakness: 'similarToEmail',
    });
  });

  // The counterweight: a fragment that happens to appear inside a long
  // passphrase leaves plenty of secret around it.
  it('accepts a passphrase that merely contains a short name', () => {
    const eric: UserIdentity = {
      email: 'eric@example.com',
      firstName: 'Eric',
      lastName: 'Smith',
    };
    expect(checkPassword('generic thunder quilt', eric)).toEqual({ acceptable: true });
  });

  it('accepts a password that merely opens with the user name', () => {
    const tester: UserIdentity = {
      email: 'test-navigation-1754000000000@hook0.local',
      firstName: 'Test',
      lastName: 'User',
    };
    expect(checkPassword('TestPassword123!1754000000000', tester)).toEqual({ acceptable: true });
  });

  it('accepts a passphrase unrelated to the user', () => {
    expect(checkPassword('correct horse battery staple', identity)).toEqual({ acceptable: true });
  });

  it('accepts a password made only of punctuation, which the API bounds instead', () => {
    expect(checkPassword('!!!!!!!!!!!!', identity)).toEqual({ acceptable: true });
  });

  it('does not let a very short name reject unrelated passwords', () => {
    const shortNames: UserIdentity = {
      email: 'l.i@example.com',
      firstName: 'Li',
      lastName: 'Ito',
    };
    expect(checkPassword('quilt lantern harbour', shortNames)).toEqual({ acceptable: true });
  });

  it('handles an address with no local part separator', () => {
    const malformed: UserIdentity = { email: 'nobody', firstName: '', lastName: '' };
    expect(checkPassword('nobody', malformed)).toEqual({
      acceptable: false,
      weakness: 'similarToEmail',
    });
  });
});

// Pinning the fold was not enough: two implementations can fold identically and
// still reach different verdicts. These pin the decision, and the API asserts
// the same file.
describe('the identity rules match the vectors shared with the API', () => {
  it.each(sharedVectors)('$why', ({ password, email, firstName, lastName, verdict }) => {
    const result = checkPassword(password, { email, firstName, lastName });
    const actual = result.acceptable ? 'acceptable' : result.weakness;
    expect(actual).toBe(verdict);
  });
});

describe('foldIdentity', () => {
  // These vectors are the contract with `fold_identity` in the API: the two
  // halves of the policy must see the same password, or the form and the server
  // disagree on what is acceptable.
  it.each([
    ['P@ssw0rd!', 'passwordi'],
    ['Jordan.Rivera+hook0@example.com', 'jordanriverahookoaexamplecom'],
    ['2026', '2o26'],
    ['john.doe2026', 'johndoe2o26'],
  ])('folds %s into %s', (value, expected) => {
    expect(foldIdentity(value)).toBe(expected);
  });

  // A substitution table and a character class that disagree would silently
  // splice "undefined" into every folded value.
  it('never produces a substitution it does not know', () => {
    fc.assert(
      fc.property(fc.string({ unit: 'binary' }), (value) => {
        expect(foldIdentity(value)).not.toContain('undefined');
      }),
      { numRuns: 500, seed: 20260806 }
    );
  });
});

describe('password policy invariants', () => {
  const emailArbitrary = fc
    .tuple(
      fc.stringMatching(/^[a-zA-Z0-9._+-]{1,40}$/),
      fc.stringMatching(/^[a-z]{1,20}\.[a-z]{2,6}$/)
    )
    .map(([localPart, domain]) => `${localPart}@${domain}`);

  // The invariant behind the report, over every address rather than the single
  // one that was reported.
  it('never accepts a password equal to the email address', () => {
    fc.assert(
      fc.property(emailArbitrary, (email) => {
        const verdict = checkPassword(email, { email, firstName: '', lastName: '' });
        expect(verdict.acceptable).toBe(false);
      }),
      { numRuns: 500, seed: 20260806 }
    );
  });

  it('never accepts the email address typed back in another case', () => {
    fc.assert(
      fc.property(emailArbitrary, (email) => {
        const user = { email, firstName: '', lastName: '' };
        expect(checkPassword(email.toUpperCase(), user).acceptable).toBe(false);
        expect(checkPassword(email.toLowerCase(), user).acceptable).toBe(false);
      }),
      { numRuns: 500, seed: 20260806 }
    );
  });

  // The form feeds this whatever the user types, so nothing may make it throw.
  it('never throws on arbitrary input', () => {
    fc.assert(
      fc.property(
        fc.string({ unit: 'binary' }),
        fc.string({ unit: 'binary' }),
        fc.string({ unit: 'binary' }),
        fc.string({ unit: 'binary' }),
        (password, ...identity) => {
          const [email, firstName, lastName] = identity;
          expect(() => checkPassword(password, { email, firstName, lastName })).not.toThrow();
        }
      ),
      { numRuns: 500, seed: 20260806 }
    );
  });

  it('folds idempotently, which is what lets folded values be compared as-is', () => {
    fc.assert(
      fc.property(fc.string({ unit: 'binary' }), (value) => {
        const folded = foldIdentity(value);
        expect(foldIdentity(folded)).toBe(folded);
      }),
      { numRuns: 500, seed: 20260806 }
    );
  });
});

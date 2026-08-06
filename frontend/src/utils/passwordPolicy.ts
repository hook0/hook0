/**
 * Client-side half of the password policy enforced by the API
 * (`api/src/password.rs`): the part that only needs what the form already
 * knows, so the user is told before submitting rather than after.
 *
 * The API stays authoritative and runs the whole policy, including the list of
 * the most common passwords — ten thousand entries have no business in a
 * bundle. What lives here is the user-specific half: a password must not be
 * built from the account's own email address or name, which is what the
 * reported issue was about.
 */

export type UserIdentity = {
  readonly email: string;
  readonly firstName: string;
  readonly lastName: string;
};

export type PasswordWeakness = 'similarToEmail' | 'similarToName';

/**
 * The verdict on a password. A tagged union rather than an optional weakness,
 * so "acceptable" is a value the caller has to read rather than an absence it
 * could forget to handle.
 */
export type PasswordVerdict =
  | { readonly acceptable: true }
  | { readonly acceptable: false; readonly weakness: PasswordWeakness };

const ACCEPTABLE: PasswordVerdict = { acceptable: true };

/** What to tell the user about each weakness, for every form that runs the check. */
export const WEAKNESS_MESSAGE_KEYS: Readonly<Record<PasswordWeakness, string>> = {
  similarToEmail: 'validation.passwordSimilarToEmail',
  similarToName: 'validation.passwordSimilarToName',
};

/**
 * Shortest identity fragment we look for *inside* a password. Below this, a
 * short surname would reject unrelated passwords that merely happen to contain
 * those letters.
 */
const MINIMUM_FRAGMENT_LENGTH = 4;

/**
 * What must be left of a password once the part an attacker can simply look up
 * — the address, the name — is taken out of it. Below this the password *is*
 * that fragment plus decoration; above it the fragment is incidental, and a
 * passphrase is not weak merely because its owner is called Eric.
 */
const MINIMUM_REMAINDER = 8;

/** The digit and symbol substitutions people use to disguise a word. */
const LOOKALIKES: Readonly<Record<string, string>> = {
  '0': 'o',
  '1': 'i',
  '3': 'e',
  '4': 'a',
  '5': 's',
  '7': 't',
  '8': 'b',
  '@': 'a',
  $: 's',
  '!': 'i',
};

/**
 * Reduce a value to the letters a human would recognise in it, so that
 * `Jordan.Rivera+hook0@example.com` and `jordan rivera` compare on their content
 * rather than on their punctuation. Mirrors `fold_identity` in the API.
 */
export function foldIdentity(value: string): string {
  return (
    value
      .toLowerCase()
      // The class lists exactly the keys of LOOKALIKES, so the replacer always
      // finds one: no character reaches it without a substitution to return.
      .replace(/[0134578@$!]/g, (character) => LOOKALIKES[character])
      .replace(/[^\p{L}\p{N}]/gu, '')
  );
}

function localPartOf(email: string): string {
  const separator = email.indexOf('@');
  return separator === -1 ? email : email.slice(0, separator);
}

/**
 * Is this password too close to who the user is? The API runs the same check
 * (plus the rest of the policy) and has the final word.
 */
export function checkPassword(password: string, identity: UserIdentity): PasswordVerdict {
  const foldedPassword = foldIdentity(password);
  if (foldedPassword === '') {
    return ACCEPTABLE;
  }

  const localPart = localPartOf(identity.email);

  // Being *equal* to the address (or to its local part) is refused whatever the
  // length: this is the "my password is my email" case.
  const isTheAddressItself = [identity.email, localPart].some(
    (value) => foldIdentity(value) === foldedPassword
  );
  if (isTheAddressItself) {
    return { acceptable: false, weakness: 'similarToEmail' };
  }

  // Then containment: a password is refused for carrying an identity fragment
  // only when what surrounds the fragment is too little to be a secret of its
  // own.
  const fragments: ReadonlyArray<readonly [PasswordWeakness, string]> = [
    ['similarToEmail', localPart],
    ...localPart.split(/[._+-]/).map((part) => ['similarToEmail', part] as const),
    ['similarToName', identity.firstName],
    ['similarToName', identity.lastName],
  ];

  for (const [weakness, value] of fragments) {
    const folded = foldIdentity(value);
    if (folded.length < MINIMUM_FRAGMENT_LENGTH) {
      continue;
    }

    const isSimilar =
      folded.includes(foldedPassword) ||
      (foldedPassword.includes(folded) &&
        foldedPassword.length - folded.length < MINIMUM_REMAINDER);
    if (isSimilar) {
      return { acceptable: false, weakness };
    }
  }

  return ACCEPTABLE;
}

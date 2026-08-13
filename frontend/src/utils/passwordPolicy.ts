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

/**
 * What to assume until GET /instance answers with the operator's own floor.
 * Mirrors the API's shipped default (PASSWORD_MINIMUM_LENGTH), so a form shown
 * before the config lands is never more permissive than the server.
 */
export const DEFAULT_PASSWORD_MINIMUM_LENGTH = 12;

/**
 * The API's ceiling (MAXIMUM_LENGTH in api/src/password.rs), which is not
 * operator-configurable and so is mirrored rather than fetched. A passphrase
 * pasted from a password manager can exceed it, and learning that from a round
 * trip is a poor way to find out. Pinned against the API by
 * password-policy-vectors.json.
 */
export const PASSWORD_MAXIMUM_LENGTH = 100;

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
      // `\p{Alphabetic}`, not `\p{L}`: Rust's `char::is_alphanumeric` is
      // Alphabetic ∪ N, which also covers the Other_Alphabetic marks used to
      // write Devanagari, Thai, Arabic and Hebrew. With `\p{L}` this form
      // refused passwords the API accepts, for those users only.
      .replace(/[^\p{Alphabetic}\p{N}]/gu, '')
  );
}

/**
 * Count characters the way Rust's `chars().count()` does — code points, not
 * UTF-16 code units. `.length` counts an astral character twice, which moved
 * the remainder threshold and made this form accept what the API refuses.
 */
function codePointLength(value: string): number {
  return [...value].length;
}

function splitEmail(email: string): { localPart: string; domain: string } {
  const separator = email.indexOf('@');
  if (separator === -1) {
    return { localPart: email, domain: '' };
  }
  return { localPart: email.slice(0, separator), domain: email.slice(separator + 1) };
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

  const { localPart, domain } = splitEmail(identity.email);

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
  //
  // The whole address is a fragment in its own right, not just its local part.
  // Otherwise the domain — public, and often the longest half — counts as
  // leftover secret, and `someone@example.com!` walks past a rule written to
  // refuse `someone@example.com`.
  //
  // Keep this list in the same order as `rejection_for` in the API
  // (api/src/password.rs): the first match decides which reason the user is
  // shown, so a different order means the two halves blame different things.
  const fragments: ReadonlyArray<readonly [PasswordWeakness, string]> = [
    ['similarToEmail', identity.email],
    ['similarToEmail', localPart],
    ['similarToEmail', domain],
    ...localPart.split(/[._+-]/).map((part) => ['similarToEmail', part] as const),
    ['similarToName', identity.firstName],
    ['similarToName', identity.lastName],
  ];

  const passwordLength = codePointLength(foldedPassword);

  for (const [weakness, value] of fragments) {
    const folded = foldIdentity(value);
    const fragmentLength = codePointLength(folded);
    if (fragmentLength < MINIMUM_FRAGMENT_LENGTH) {
      continue;
    }

    // The password being *inside* the fragment only means something when the
    // password has enough letters left to be recognisable as part of it. A
    // password of punctuation folds to a letter or two, and almost any address
    // contains those — telling someone their pile of symbols was built from
    // their email address is both wrong and baffling.
    const isSimilar =
      (passwordLength >= MINIMUM_FRAGMENT_LENGTH && folded.includes(foldedPassword)) ||
      (foldedPassword.includes(folded) && passwordLength - fragmentLength < MINIMUM_REMAINDER);
    if (isSimilar) {
      return { acceptable: false, weakness };
    }
  }

  return ACCEPTABLE;
}

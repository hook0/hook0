//! Password policy and hashing, shared by every path that stores a user password.
//!
//! The rules follow NIST SP 800-63B: a length floor plus two blocklists — the
//! passwords everybody picks, and the words specific to this user (their email
//! address and their name) — rather than composition rules ("one uppercase, one
//! digit, one symbol") which mostly push people towards predictable
//! substitutions.
//!
//! The policy is carried by the type system: [`hash`] accepts nothing but a
//! [`Checked`], so a password cannot reach the database without going through
//! [`Checked::new`] (or the deliberate, documented [`Checked::already_established`]).
//!
//! `common_passwords.txt` is the "10k most common passwords" list from
//! [SecLists](https://github.com/danielmiessler/SecLists) (MIT licensed).

use actix_web::rt::task::spawn_blocking;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHashString, SaltString};
use argon2::{Argon2, PasswordHasher};
use std::collections::HashSet;
use std::sync::LazyLock;
use tracing::error;

use crate::problems::Hook0Problem;

/// Upper bound on an accepted password. Argon2 happily hashes megabytes, so
/// without this a request body could buy an arbitrary amount of CPU.
pub const MAXIMUM_LENGTH: usize = 100;

/// A password built from fewer distinct characters than this is a repetition
/// ("aaaaaaaaaaaa") or an alternation ("abababababab"): long enough to satisfy
/// the length floor, trivial to guess.
const MINIMUM_DISTINCT_CHARACTERS: usize = 4;

/// Shortest identity fragment we look for *inside* a password. Below this,
/// short surnames ("Li", "Ito") would reject unrelated passwords that merely
/// happen to contain those letters.
const MINIMUM_FRAGMENT_LENGTH: usize = 4;

/// What must be left of a password once the part an attacker can simply look up
/// — the address, the name — is taken out of it. Below this the password *is*
/// that fragment plus decoration; above it the fragment is incidental, and a
/// passphrase is not weak merely because its owner is called Eric.
const MINIMUM_REMAINDER: usize = 8;

/// The common passwords, each stored in the canonical form both sides of a
/// comparison are reduced to, so a disguise is stored as the word it disguises.
static COMMON_PASSWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    include_str!("common_passwords.txt")
        .lines()
        .map(|line| fold_confusables(&line.trim().to_lowercase()))
        .filter(|entry| !entry.is_empty())
        .collect()
});

/// The user-specific words a password must not be built from.
#[derive(Debug, Clone, Copy)]
pub struct UserIdentity<'a> {
    pub email: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
}

/// Why a password was refused. Each variant maps to its own documented API
/// error so a client can tell the user what to fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum Rejection {
    TooShort,
    TooLong,
    SimilarToEmail,
    SimilarToName,
    TooCommon,
    NotEnoughDistinctCharacters,
}

impl Rejection {
    /// The length floor is instance configuration, so only the caller knows the
    /// number to put in front of the user.
    pub fn into_problem(self, minimum_length: u8) -> Hook0Problem {
        match self {
            Self::TooShort => Hook0Problem::PasswordTooShort(minimum_length),
            Self::TooLong => Hook0Problem::PasswordTooLong,
            Self::SimilarToEmail => Hook0Problem::PasswordSimilarToEmail,
            Self::SimilarToName => Hook0Problem::PasswordSimilarToName,
            Self::TooCommon => Hook0Problem::PasswordTooCommon,
            Self::NotEnoughDistinctCharacters => Hook0Problem::PasswordNotDiverseEnough,
        }
    }
}

/// A password that satisfied the policy.
#[derive(Debug, Clone, Copy)]
pub struct Checked<'a>(&'a str);

impl<'a> Checked<'a> {
    /// Run the policy. Every field of `identity` is compared case- and
    /// lookalike-insensitively, so `Bob@example.com` is no more acceptable a
    /// password than `b0b@example.com`.
    pub fn new(
        password: &'a str,
        minimum_length: u8,
        identity: &UserIdentity<'_>,
    ) -> Result<Self, Rejection> {
        // Characters, not bytes: "éééééééééééé" is twelve characters a user
        // typed but twenty-four bytes, and counting bytes would let a password
        // shorter than the floor through.
        let length = password.chars().count();
        if length < usize::from(minimum_length) {
            return Err(Rejection::TooShort);
        }
        if length > MAXIMUM_LENGTH {
            return Err(Rejection::TooLong);
        }

        if password.chars().collect::<HashSet<_>>().len() < MINIMUM_DISTINCT_CHARACTERS
            || is_repetition(password)
        {
            return Err(Rejection::NotEnoughDistinctCharacters);
        }

        if let Some(rejection) = identity.rejection_for(password) {
            return Err(rejection);
        }

        if is_common(password) {
            return Err(Rejection::TooCommon);
        }

        Ok(Self(password))
    }

    /// A credential another authority already accepted: the Keycloak import
    /// replays the very password the user just logged in with. Re-running the
    /// policy here would lock a legitimate user out of their own account
    /// mid-migration, so the policy applies to their *next* password instead.
    #[cfg(any(feature = "migrate-users-from-keycloak", test))]
    pub fn already_established(password: &'a str) -> Self {
        Self(password)
    }
}

/// Hash a password with Argon2, off the async runtime — hashing is deliberately
/// slow and would otherwise block the whole worker thread.
pub async fn hash(password: Checked<'_>) -> Result<PasswordHashString, Hook0Problem> {
    let password = password.0.to_owned();

    spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                error!("Error trying to hash user password: {e}");
                Hook0Problem::InternalServerError
            })
            .map(|h| h.serialize())
    })
    .await
    .map_err(|e| {
        error!("Failed to run password hashing task: {e}");
        Hook0Problem::InternalServerError
    })?
}

impl UserIdentity<'_> {
    /// The reason this password is too close to who the user is, if it is.
    fn rejection_for(&self, password: &str) -> Option<Rejection> {
        let folded_password = fold_identity(password);
        if folded_password.is_empty() {
            return None;
        }

        let (local_part, domain) = match self.email.split_once('@') {
            Some((local_part, domain)) => (local_part, domain),
            None => (self.email, ""),
        };

        // Being *equal* to the address (or to its local part) is refused
        // whatever the length: this is the "my password is my email" case.
        for value in [self.email, local_part] {
            if fold_identity(value) == folded_password {
                return Some(Rejection::SimilarToEmail);
            }
        }

        // Then containment: a password is refused for carrying an identity
        // fragment only when what surrounds the fragment is too little to be a
        // secret of its own.
        //
        // The whole address is a fragment in its own right, not just its local
        // part. Otherwise the domain — public, and often the longest half —
        // counts as leftover secret, and `someone@example.com!` walks past a
        // rule written to refuse `someone@example.com`.
        //
        // Keep this list in the same order as `checkPassword` in the frontend
        // (frontend/src/utils/passwordPolicy.ts): the first match decides which
        // reason the user is shown, so a different order means the two halves
        // blame different things for the same password.
        let mut fragments = vec![
            (Rejection::SimilarToEmail, self.email),
            (Rejection::SimilarToEmail, local_part),
            (Rejection::SimilarToEmail, domain),
        ];
        // A local part is usually "first.last" or "first_last"; each half is as
        // guessable as the whole.
        fragments.extend(
            local_part
                .split(['.', '_', '-', '+'])
                .map(|part| (Rejection::SimilarToEmail, part)),
        );
        fragments.push((Rejection::SimilarToName, self.first_name));
        fragments.push((Rejection::SimilarToName, self.last_name));

        let password_length = folded_password.chars().count();

        fragments.into_iter().find_map(|(rejection, value)| {
            let folded = fold_identity(value);
            let fragment_length = folded.chars().count();
            if fragment_length < MINIMUM_FRAGMENT_LENGTH {
                return None;
            }

            let is_similar = folded.contains(&folded_password)
                || (folded_password.contains(&folded)
                    && password_length.saturating_sub(fragment_length) < MINIMUM_REMAINDER);
            is_similar.then_some(rejection)
        })
    }
}

/// Is the password a shorter unit typed several times? Counting distinct
/// characters only catches a unit of one or two ("aaaa", "abab"); `abcdabcdabcd`
/// clears both the length floor and the character count while carrying the
/// entropy of four characters. Padding is stripped first, or one trailing digit
/// would break the periodicity and buy the whole trick back.
fn is_repetition(password: &str) -> bool {
    padding_cores(password)
        .into_iter()
        .filter(|core| core.chars().count() >= MINIMUM_FRAGMENT_LENGTH)
        .any(is_repeated_unit)
}

fn is_repeated_unit(candidate: &str) -> bool {
    let characters = candidate.chars().collect::<Vec<_>>();
    let length = characters.len();

    (1..=length / 2).any(|unit| {
        length % unit == 0
            && characters
                .chunks(unit)
                .all(|chunk| chunk == &characters[..unit])
    })
}

/// Every word the padding at either end could be hiding.
///
/// Padding is digits and punctuation, but the glyph where it meets the word is
/// ambiguous: in "chicag02026!" the zero belongs to the word and the rest is
/// decoration, in "letmein2026!" none of it does. Guessing that boundary is
/// what let both tricks through — trimming first ate the zero of "chicago",
/// folding first turned "2026" into "2o26" and left nothing to trim. So the
/// boundary is not guessed: every cut inside each padding run is offered, and
/// the caller tests them all. Bounded by the password's own length ceiling.
fn padding_cores(value: &str) -> Vec<&str> {
    let is_padding = |c: char| c.is_ascii_digit() || c.is_ascii_punctuation();

    let boundaries = value
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(value.len()))
        .collect::<Vec<_>>();
    let length = boundaries.len() - 1;

    let leading = value.chars().take_while(|c| is_padding(*c)).count();
    let trailing = value.chars().rev().take_while(|c| is_padding(*c)).count();

    let mut cores = Vec::new();
    for start in 0..=leading {
        for end in start.max(length - trailing)..=length {
            cores.push(&value[boundaries[start]..boundaries[end]]);
        }
    }
    cores
}

/// Fold the digit and symbol substitutions people use to disguise a word
/// ("P@ssw0rd" becomes "password"), so a disguise is recognised as the word.
/// Shared with the frontend, character for character — see
/// `password-policy-vectors.json`.
fn fold_lookalikes(lowercased: &str) -> String {
    lowercased
        .chars()
        .map(|c| match c {
            '0' => 'o',
            '1' => 'i',
            '3' => 'e',
            '4' => 'a',
            '5' => 's',
            '7' => 't',
            '8' => 'b',
            '@' => 'a',
            '$' => 's',
            '!' => 'i',
            other => other,
        })
        .collect()
}

/// Collapse each group of glyphs people write for one another onto a single
/// representative, so that comparing two strings ignores which member of the
/// group was typed.
///
/// This is what the blocklist is matched on, and it is deliberately blunter
/// than `fold_lookalikes`: "million", "m1ll1on" and "mi11ion" all reduce to the
/// same thing, where a fold that picks one reading per glyph can only recover
/// the disguises that happen to agree with its choice. Not used for the
/// identity rules — those are shared with the frontend and must fold, not
/// collapse.
fn fold_confusables(lowercased: &str) -> String {
    lowercased
        .chars()
        .map(|c| match c {
            'i' | 'l' | '1' | '!' | '|' => '1',
            'o' | '0' => 'o',
            'e' | '3' => 'e',
            'a' | '4' | '@' => 'a',
            's' | '5' | '$' => 's',
            't' | '7' => 't',
            'b' | '8' => 'b',
            other => other,
        })
        .collect()
}

/// Reduce a value to the letters a human would recognise in it, so that
/// `Jordan.Rivera+hook0@example.com` and `jordan rivera` compare on their content
/// rather than on their punctuation.
fn fold_identity(value: &str) -> String {
    fold_lookalikes(&value.to_lowercase())
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Is this one of the passwords everybody picks — possibly disguised, possibly
/// padded to reach the length floor? Padding a word and disguising its letters
/// are the two cheapest ways to stretch a common password into a compliant one,
/// so neither is allowed to hide it.
fn is_common(password: &str) -> bool {
    let lowercased = password.to_lowercase();

    padding_cores(&lowercased)
        .into_iter()
        .filter(|core| core.chars().count() >= MINIMUM_FRAGMENT_LENGTH)
        .any(|core| COMMON_PASSWORDS.contains(&fold_confusables(core)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const MINIMUM_LENGTH: u8 = 12;

    fn identity<'a>() -> UserIdentity<'a> {
        UserIdentity {
            email: "jordanrivera801@example.com",
            first_name: "Jordan",
            last_name: "Rivera",
        }
    }

    fn check(password: &str) -> Result<(), Rejection> {
        Checked::new(password, MINIMUM_LENGTH, &identity()).map(|_| ())
    }

    /// The reported vulnerability: an account whose password is its own email
    /// address. Everything else in this module exists to keep this test green.
    #[test]
    fn a_password_equal_to_the_email_address_is_refused() {
        assert_eq!(
            check("jordanrivera801@example.com"),
            Err(Rejection::SimilarToEmail)
        );
    }

    /// Why the length floor that used to be the whole policy was not enough,
    /// stated as an assertion so nobody reintroduces it as the only rule: the
    /// reported password is long enough, and still one an attacker guesses
    /// first.
    #[test]
    fn a_length_floor_alone_would_have_accepted_the_reported_password() {
        let reported = "jordanrivera801@example.com";
        assert!(reported.chars().count() >= usize::from(MINIMUM_LENGTH));
        assert!(check(reported).is_err());
    }

    /// Decorating the address with a character or two must not walk past a rule
    /// written to refuse the address. The domain is public: it is not the part
    /// that makes a password a secret.
    #[test]
    fn padding_the_email_address_does_not_get_it_past_the_rule() {
        for password in [
            "jordanrivera801@example.com!",
            "jordanrivera801@example.com1",
            "xjordanrivera801@example.com",
            "JORDANRIVERA801@EXAMPLE.COM!",
            "..jordanrivera801@example.com..",
        ] {
            assert_eq!(
                check(password),
                Err(Rejection::SimilarToEmail),
                "accepted a decorated email address: {password}"
            );
        }
    }

    /// The domain on its own is guessable from the address, so a password that
    /// is mostly the domain is no better than one that is mostly the local part.
    #[test]
    fn a_password_that_is_mostly_the_domain_is_refused() {
        let identity = UserIdentity {
            email: "bob@acmecorporation.com",
            first_name: "Bob",
            last_name: "Smith",
        };
        assert_eq!(
            Checked::new("acmecorporation.com2026", MINIMUM_LENGTH, &identity).map(|_| ()),
            Err(Rejection::SimilarToEmail)
        );
    }

    /// Counting distinct characters only catches a unit of one or two. A longer
    /// unit typed several times is just as guessable.
    #[test]
    fn a_repeated_unit_is_refused_however_long_the_unit() {
        for password in [
            "abcdabcdabcd",
            "123412341234",
            "12341234123412341234",
            "abcdefabcdefabcdef",
            "hook0hook0hook0hook0",
        ] {
            assert_eq!(
                check(password),
                Err(Rejection::NotEnoughDistinctCharacters),
                "accepted a repeated unit: {password}"
            );
        }
    }

    /// Prefixing a year is as common as suffixing one, so padding is stripped
    /// from both ends before the blocklist is consulted.
    #[test]
    fn padding_a_common_password_at_either_end_does_not_help() {
        for password in [
            "letmein2026!",
            "2026letmein!",
            "2026password",
            "!!!password!!",
            "..qwerty12345",
        ] {
            assert_eq!(
                check(password),
                Err(Rejection::TooCommon),
                "accepted a padded common password: {password}"
            );
        }
    }

    /// `1` stands in for both "l" and "i". Reading it only as "i" let every
    /// common password containing an "l" through — a third of the blocklist.
    #[test]
    fn writing_l_as_one_does_not_hide_a_common_password() {
        for password in [
            "1etmein2026!",
            "baseba112026!",
            "f00tba112026!",
            "michae12026!",
            "char1ie2026!",
            "miche11e2026!",
        ] {
            assert_eq!(
                check(password),
                Err(Rejection::TooCommon),
                "accepted a common password written with ones: {password}"
            );
        }
    }

    /// One character of decoration used to buy the whole trick back: it broke
    /// the periodicity, and a repeated unit was only ever tested on the whole
    /// string.
    #[test]
    fn padding_a_repeated_unit_does_not_hide_it() {
        for password in [
            "abcdabcdabcd99",
            "abcabcabcabc!",
            "1234123412340",
            "hook0hook0hook0hook01",
            "12ababababababab",
            "azazazazazaz09",
        ] {
            assert_eq!(
                check(password),
                Err(Rejection::NotEnoughDistinctCharacters),
                "accepted a padded repeated unit: {password}"
            );
        }
    }

    /// The two glyphs the padding and the word fight over. Reading `1` as a
    /// letter everywhere turned the padding's own ones into letters, and
    /// trimming before folding ate the `0` that was the "o" of "chicago" —
    /// so where the word ends is searched for, never guessed.
    #[test]
    fn a_disguise_that_reaches_into_the_padding_does_not_hide_a_common_password() {
        for password in [
            // The padding carries a one of its own.
            "1etmein12345",
            "baseba1112345",
            "michae112345",
            "char1ie12345",
            // The word ends on a glyph the padding would claim.
            "chicag02026!",
            "sc0rpi02026!",
        ] {
            assert_eq!(
                check(password),
                Err(Rejection::TooCommon),
                "accepted a disguised common password: {password}"
            );
        }
    }

    /// The blocklist is 10000 entries and the tests above pin a dozen. Sweeping
    /// the real list is what turns "these six disguises are caught" into "this
    /// disguise is caught, for every word we ship" — the two escapes above were
    /// found by a sweep and would have been born caught with one.
    #[test]
    fn no_shipped_common_password_survives_being_disguised_and_padded() {
        let words = include_str!("common_passwords.txt")
            .lines()
            .map(str::trim)
            .filter(|word| {
                word.chars().count() >= MINIMUM_FRAGMENT_LENGTH
                    && word.chars().all(|c| c.is_ascii_lowercase())
            })
            .collect::<Vec<_>>();
        assert!(words.len() > 1000, "the blocklist did not load: {words:?}");

        let disguise = |word: &str| {
            word.chars()
                .map(|c| match c {
                    'o' => '0',
                    'i' | 'l' => '1',
                    'a' => '@',
                    'e' => '3',
                    's' => '5',
                    other => other,
                })
                .collect::<String>()
        };

        for word in words {
            for candidate in [
                format!("{word}12345"),
                format!("{}12345", disguise(word)),
                format!("{}2026!", disguise(word)),
                format!("2026!{}", disguise(word)),
                format!("..{}..", disguise(word)),
            ] {
                assert!(
                    is_common(&candidate),
                    "{candidate:?} hides the common password {word:?}"
                );
            }
        }
    }

    #[test]
    fn the_email_address_is_refused_whatever_its_case() {
        assert_eq!(
            check("JordanRivera801@Example.com"),
            Err(Rejection::SimilarToEmail)
        );
    }

    #[test]
    fn the_local_part_of_the_email_address_is_refused() {
        assert_eq!(check("jordanrivera801"), Err(Rejection::SimilarToEmail));
    }

    #[test]
    fn a_password_built_around_the_email_address_is_refused() {
        assert_eq!(
            check("xx-jordanrivera801-xx"),
            Err(Rejection::SimilarToEmail)
        );
    }

    #[test]
    fn a_password_that_is_mostly_the_local_part_is_refused() {
        let identity = UserIdentity {
            email: "john.doe@example.com",
            first_name: "",
            last_name: "",
        };
        assert_eq!(
            Checked::new("john.doe2026", MINIMUM_LENGTH, &identity).map(|_| ()),
            Err(Rejection::SimilarToEmail)
        );
    }

    /// A local part is usually "first.last", and each half is as guessable as
    /// the whole. Checked without the length floor in the way, since the point
    /// here is the split rather than the length.
    #[test]
    fn each_half_of_a_dotted_local_part_is_refused() {
        let identity = UserIdentity {
            email: "john.doe@example.com",
            first_name: "",
            last_name: "",
        };
        assert_eq!(
            Checked::new("john", 0, &identity).map(|_| ()),
            Err(Rejection::SimilarToEmail)
        );
    }

    /// The counterweight to the rule above: a fragment that happens to appear
    /// inside a long passphrase leaves plenty of secret around it, and refusing
    /// it would only teach users to fight the form.
    #[test]
    fn a_passphrase_is_not_weak_merely_for_containing_a_short_name() {
        let identity = UserIdentity {
            email: "eric@example.com",
            first_name: "Eric",
            last_name: "Smith",
        };
        assert_eq!(
            Checked::new("generic thunder quilt", MINIMUM_LENGTH, &identity).map(|_| ()),
            Ok(())
        );
    }

    /// The shape of credential the end-to-end suite registers with: it opens
    /// with the account's first name, and everything after it is the actual
    /// secret. Refusing it would break every browser test for no security gain.
    #[test]
    fn a_password_that_merely_opens_with_the_user_name_is_accepted() {
        let identity = UserIdentity {
            email: "test-navigation-1754000000000@hook0.local",
            first_name: "Test",
            last_name: "User",
        };
        assert_eq!(
            Checked::new("TestPassword123!1754000000000", MINIMUM_LENGTH, &identity).map(|_| ()),
            Ok(())
        );
    }

    #[test]
    fn a_password_built_around_the_user_name_is_refused() {
        assert_eq!(check("rivera-is-here"), Err(Rejection::SimilarToName));
    }

    #[test]
    fn lookalike_substitutions_do_not_hide_the_email_address() {
        assert_eq!(
            check("j0rd4nrivera801@example.com"),
            Err(Rejection::SimilarToEmail)
        );
    }

    #[test]
    fn a_common_password_is_refused() {
        assert_eq!(check("qwertyuiop12"), Err(Rejection::TooCommon));
    }

    #[test]
    fn padding_a_common_password_to_reach_the_length_floor_does_not_help() {
        assert_eq!(check("letmein2026!"), Err(Rejection::TooCommon));
        assert_eq!(check("password1234"), Err(Rejection::TooCommon));
    }

    #[test]
    fn disguising_a_common_password_does_not_help() {
        assert_eq!(check("P@ssw0rd2026"), Err(Rejection::TooCommon));
    }

    #[test]
    fn a_repeated_character_is_refused() {
        assert_eq!(
            check("aaaaaaaaaaaaaa"),
            Err(Rejection::NotEnoughDistinctCharacters)
        );
        assert_eq!(
            check("abababababab"),
            Err(Rejection::NotEnoughDistinctCharacters)
        );
    }

    #[test]
    fn a_short_password_is_refused() {
        assert_eq!(check("Sh0rt!"), Err(Rejection::TooShort));
    }

    #[test]
    fn length_counts_characters_rather_than_bytes() {
        // Ten characters, twenty bytes: long enough for a byte count, too short
        // for the user who typed it.
        assert_eq!(check("éàüßñéàüßñ"), Err(Rejection::TooShort));
    }

    #[test]
    fn an_over_long_password_is_refused() {
        let password = "diceware-correct-horse-".repeat(10);
        assert!(password.chars().count() > MAXIMUM_LENGTH);
        assert_eq!(check(&password), Err(Rejection::TooLong));
    }

    #[test]
    fn a_passphrase_unrelated_to_the_user_is_accepted() {
        assert_eq!(check("correct horse battery staple"), Ok(()));
        assert_eq!(check("Tr0ub4dor&3-quilt-lantern"), Ok(()));
    }

    /// A user whose surname happens to be short must not have every password
    /// containing those letters refused.
    #[test]
    fn a_very_short_name_does_not_reject_unrelated_passwords() {
        let identity = UserIdentity {
            email: "l.i@example.com",
            first_name: "Li",
            last_name: "Ito",
        };
        assert_eq!(
            Checked::new("quilt lantern harbour", MINIMUM_LENGTH, &identity).map(|_| ()),
            Ok(())
        );
    }

    #[test]
    fn an_already_established_credential_skips_the_policy() {
        // The Keycloak import must never lock a user out of their own account.
        assert_eq!(Checked::already_established("123456").0, "123456");
    }

    /// The shared contract with the frontend. Pinning the *fold* was not enough:
    /// two implementations can fold identically and still reach different
    /// verdicts, which is exactly how the form and the server drifted apart on
    /// astral characters, on Other_Alphabetic marks, and on which rule to blame.
    /// These vectors pin the decision itself, and the same file drives
    /// `passwordPolicy.test.ts`.
    #[test]
    fn the_identity_rules_match_the_vectors_shared_with_the_frontend() {
        let shared = include_str!("../../password-policy-vectors.json");
        let parsed = serde_json::from_str::<serde_json::Value>(shared).expect("shared vectors");
        let vectors = parsed["vectors"].as_array().expect("vectors array");
        assert!(!vectors.is_empty());

        for vector in vectors {
            let text = |key: &str| vector[key].as_str().unwrap_or_default().to_owned();
            let (password, expected) = (text("password"), text("verdict"));
            let (email, first_name, last_name) =
                (text("email"), text("firstName"), text("lastName"));

            let identity = UserIdentity {
                email: &email,
                first_name: &first_name,
                last_name: &last_name,
            };
            let verdict = match identity.rejection_for(&password) {
                Some(Rejection::SimilarToEmail) => "similarToEmail",
                Some(Rejection::SimilarToName) => "similarToName",
                Some(other) => panic!("identity rules returned {other:?} for {password:?}"),
                None => "acceptable",
            };

            assert_eq!(
                verdict,
                expected,
                "verdict for {password:?}: {}",
                text("why")
            );
        }
    }

    /// A refused password is only useful to the user if the form can say which
    /// rule refused it, next to the field. That needs both halves to agree on
    /// the set of error ids that mean "the password was refused" rather than
    /// "the request failed" — a reason added on one side alone would reach the
    /// user as an unexplained toast.
    #[test]
    fn every_rejection_reason_is_listed_in_the_shared_problem_ids() {
        use strum::IntoEnumIterator;

        let shared = include_str!("../../password-policy-vectors.json");
        let parsed = serde_json::from_str::<serde_json::Value>(shared).expect("shared vectors");
        let listed = parsed["rejectionProblemIds"]
            .as_array()
            .expect("rejectionProblemIds array")
            .iter()
            .map(|id| id.as_str().unwrap_or_default().to_owned())
            .collect::<HashSet<_>>();

        let produced = Rejection::iter()
            .map(|rejection| rejection.into_problem(MINIMUM_LENGTH).to_string())
            .collect::<HashSet<_>>();

        assert_eq!(produced, listed);

        // The ceiling is not operator-configurable, so the frontend mirrors it
        // rather than fetching it. Raising it here alone would let a form
        // refuse a passphrase the API accepts.
        assert_eq!(
            parsed["maximumLength"].as_u64(),
            Some(MAXIMUM_LENGTH as u64)
        );
    }

    /// These vectors are the contract with `foldIdentity` in the frontend: the
    /// two halves of the policy must see the same password, or the form and the
    /// server disagree on what is acceptable.
    #[test]
    fn folding_matches_the_vectors_the_frontend_pins() {
        for (value, expected) in [
            ("P@ssw0rd!", "passwordi"),
            (
                "Jordan.Rivera+hook0@example.com",
                "jordanriverahookoaexamplecom",
            ),
            ("2026", "2o26"),
            ("john.doe2026", "johndoe2o26"),
        ] {
            assert_eq!(fold_identity(value), expected, "folding {value}");
        }
    }

    /// Passwords are compared by their content, not their punctuation, so
    /// folding twice must not fold further.
    #[test]
    fn folding_an_identity_is_idempotent() {
        let folded = fold_identity("Jordan.Rivera+hook0@example.com");
        assert_eq!(fold_identity(&folded), folded);
    }

    proptest! {
        /// The invariant behind the report, over every address rather than the
        /// single one that was reported: a password equal to the user's email
        /// address is never accepted, whatever that address looks like.
        #[test]
        fn a_password_equal_to_the_email_address_is_never_accepted(
            local_part in "[a-zA-Z0-9._+-]{1,40}",
            domain in "[a-z]{1,20}\\.[a-z]{2,6}",
            minimum_length in 0u8..=40,
        ) {
            let email = format!("{local_part}@{domain}");
            let identity = UserIdentity { email: &email, first_name: "", last_name: "" };
            prop_assert!(Checked::new(&email, minimum_length, &identity).is_err());
        }

        /// Same invariant, with the address typed back in a different case.
        #[test]
        fn the_email_address_is_never_accepted_in_another_case(
            local_part in "[a-zA-Z0-9._+-]{1,40}",
            domain in "[a-z]{1,20}\\.[a-z]{2,6}",
        ) {
            let email = format!("{local_part}@{domain}");
            let identity = UserIdentity { email: &email, first_name: "", last_name: "" };
            prop_assert!(Checked::new(&email.to_uppercase(), 0, &identity).is_err());
            prop_assert!(Checked::new(&email.to_lowercase(), 0, &identity).is_err());
        }

        /// Anything accepted sits inside the announced bounds, so no caller has
        /// to bound the password again before hashing it.
        #[test]
        fn an_accepted_password_stays_within_its_bounds(
            password in ".{0,200}",
            minimum_length in 0u8..=40,
        ) {
            let identity = UserIdentity {
                email: "someone@example.com",
                first_name: "Some",
                last_name: "One",
            };
            if Checked::new(&password, minimum_length, &identity).is_ok() {
                let length = password.chars().count();
                prop_assert!(length >= usize::from(minimum_length));
                prop_assert!(length <= MAXIMUM_LENGTH);

                // Asserting the length alone let the other four gates drift:
                // `is_repetition` shipped with no property at all, and grew a
                // bypass nobody sampled.
                prop_assert!(
                    password.chars().collect::<HashSet<_>>().len()
                        >= MINIMUM_DISTINCT_CHARACTERS
                );
                prop_assert!(!is_repetition(&password));
                prop_assert!(!is_common(&password));
            }
        }

        /// The rule the whole fix exists for, stated as an invariant rather
        /// than as a list of addresses: whatever is accepted, you cannot read
        /// the account's address out of it and have little left over. Both
        /// email properties above feed the address in whole, so they only ever
        /// exercise the equality short-circuit — this one exercises the
        /// containment path that does the actual work.
        #[test]
        fn an_accepted_password_is_not_the_email_address_with_decoration(
            local_part in "[a-z]{3,20}",
            domain in "[a-z]{3,10}\\.[a-z]{2,4}",
            decoration in ".{0,20}",
        ) {
            let email = format!("{local_part}@{domain}");
            let identity = UserIdentity {
                email: &email,
                first_name: "Some",
                last_name: "One",
            };
            let password = format!("{email}{decoration}");

            if Checked::new(&password, 0, &identity).is_ok() {
                let folded_password = fold_identity(&password);
                let folded_email = fold_identity(&email);
                prop_assert!(
                    !folded_password.contains(&folded_email)
                        || folded_password.chars().count()
                            .saturating_sub(folded_email.chars().count())
                            >= MINIMUM_REMAINDER
                );
            }
        }

        /// The policy reads untrusted request bodies: no input may make it
        /// panic, however exotic — control characters, lone surrogicates
        /// escapes, huge strings, empty identities.
        #[test]
        fn the_policy_never_panics_on_arbitrary_input(
            password in ".{0,500}",
            email in ".{0,200}",
            first_name in ".{0,100}",
            last_name in ".{0,100}",
            minimum_length in 0u8..=255,
        ) {
            let identity = UserIdentity {
                email: &email,
                first_name: &first_name,
                last_name: &last_name,
            };
            let _ = Checked::new(&password, minimum_length, &identity);
        }

        /// Folding is idempotent for every input, which is what lets the
        /// blocklist be folded once at load time and compared as-is.
        #[test]
        fn folding_is_idempotent_for_any_input(value in ".{0,200}") {
            let folded = fold_identity(&value);
            prop_assert_eq!(fold_identity(&folded), folded);
        }
    }
}

//! Where a signup came from, as a bounded label.
//!
//! The browser derives a label from the referrer and the campaign parameters
//! of the page the visitor registered on, and sends it with the registration
//! payload. That input is untrusted, so it is mapped here onto a closed
//! vocabulary before it reaches the database: anything that does not match the
//! grammar becomes [`UNKNOWN`], which is also what an older client that sends
//! nothing at all ends up storing.
//!
//! The grammar is duplicated as a CHECK constraint on `iam.user.signup_channel`
//! (migration `20260819120000_add_signup_channel`). Keep the two in step: the
//! constraint is what stops a future caller from writing free text into a
//! column whose whole value is that it can be grouped by.
//!
//! No referrer URL, query string or identifier is kept — only which family of
//! source it belonged to.

/// Label stored when nothing usable was sent, or when what was sent does not
/// match the grammar. Also the column default, so rows predating this capture
/// read the same way.
pub const UNKNOWN: &str = "unknown";

/// The visitor arrived with no referrer at all (typed the address, a
/// bookmark, a link from a native application).
const DIRECT: &str = "direct";

/// Source families accepted before the `:` separator. A label whose family is
/// not one of these is dropped rather than stored: the point of the column is
/// that `GROUP BY signup_channel` stays readable.
const FAMILIES: [&str; 7] = [
    "ads", "organic", "ai", "social", "referral", "campaign",
    // Answered by the person signing up, when nothing could be detected. Kept
    // as its own family so a declaration is never counted as an observation.
    "declared",
];

/// Longest accepted suffix after the `:`. Hostnames stay well under this, and
/// the bound is what keeps an oversized payload from reaching the DB CHECK.
const MAX_SUFFIX_LEN: usize = 64;

/// Map an untrusted channel label onto the stored vocabulary.
///
/// Surrounding whitespace is trimmed and ASCII case is folded, because a label
/// that differs only by case would split its own group. Everything else is
/// accepted as-is or replaced by [`UNKNOWN`] — never repaired, so a client bug
/// shows up as a bucket of unknowns rather than as plausible-looking data.
pub fn normalize(raw: Option<&str>) -> String {
    match raw {
        Some(value) => normalize_str(value),
        None => UNKNOWN.to_string(),
    }
}

fn normalize_str(value: &str) -> String {
    let label = value.trim().to_ascii_lowercase();

    if label == UNKNOWN || label == DIRECT {
        return label;
    }

    match label.split_once(':') {
        Some((family, suffix)) if FAMILIES.contains(&family) && is_valid_suffix(suffix) => label,
        _ => UNKNOWN.to_string(),
    }
}

/// A suffix is a short, lowercase slug: a search engine, a network, or a
/// hostname stripped of its scheme and path by the browser.
fn is_valid_suffix(suffix: &str) -> bool {
    let mut chars = suffix.chars();

    match chars.next() {
        Some(first) if first.is_ascii_alphanumeric() => {}
        _ => return false,
    }

    suffix.len() <= MAX_SUFFIX_LEN
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_labels_of_every_accepted_family() {
        for label in [
            "ads:google",
            "organic:google",
            "organic:duckduckgo",
            "ai:chatgpt",
            "social:linkedin",
            "referral:openalternative.co",
            "campaign:newsletter",
            "declared:friend",
        ] {
            assert_eq!(normalize(Some(label)), label);
        }
    }

    #[test]
    fn keeps_the_two_suffix_less_labels() {
        assert_eq!(normalize(Some("direct")), "direct");
        assert_eq!(normalize(Some(UNKNOWN)), UNKNOWN);
    }

    #[test]
    fn folds_case_and_trims_so_one_source_is_one_group() {
        assert_eq!(normalize(Some("  Organic:Google  ")), "organic:google");
        assert_eq!(normalize(Some("DIRECT")), "direct");
    }

    #[test]
    fn missing_empty_or_blank_input_reads_as_unknown() {
        assert_eq!(normalize(None), UNKNOWN);
        assert_eq!(normalize(Some("")), UNKNOWN);
        assert_eq!(normalize(Some("   ")), UNKNOWN);
    }

    #[test]
    fn refuses_an_unlisted_family() {
        assert_eq!(normalize(Some("internal:staff")), UNKNOWN);
        assert_eq!(normalize(Some("organics:google")), UNKNOWN);
    }

    #[test]
    fn refuses_free_text_and_injection_attempts() {
        for label in [
            "referral:evil.com'; DROP TABLE iam.user; --",
            "referral:<script>alert(1)</script>",
            "referral:example.com/path?utm=x",
            "referral:user@example.com",
            "referral: example.com",
            "organic:google organic:bing",
            "a whole sentence about where I came from",
        ] {
            assert_eq!(normalize(Some(label)), UNKNOWN, "accepted {label:?}");
        }
    }

    #[test]
    fn refuses_a_malformed_suffix() {
        assert_eq!(normalize(Some("referral:")), UNKNOWN);
        assert_eq!(normalize(Some("referral:.example.com")), UNKNOWN);
        assert_eq!(normalize(Some("referral:-example.com")), UNKNOWN);
        assert_eq!(normalize(Some(":google")), UNKNOWN);
    }

    #[test]
    fn refuses_a_suffix_longer_than_the_column_accepts() {
        let at_limit = format!("referral:{}", "a".repeat(MAX_SUFFIX_LEN));
        assert_eq!(normalize(Some(&at_limit)), at_limit);

        let over_limit = format!("referral:{}", "a".repeat(MAX_SUFFIX_LEN + 1));
        assert_eq!(normalize(Some(&over_limit)), UNKNOWN);
    }

    /// Every label this module can emit has to satisfy the DB CHECK, otherwise
    /// a signup fails on an INSERT instead of being recorded as unknown.
    #[test]
    fn every_emitted_label_matches_the_database_grammar() {
        let samples = [
            None,
            Some(""),
            Some("direct"),
            Some("Organic:Google"),
            Some("referral:openalternative.co"),
            Some("referral:evil.com'; DROP TABLE iam.user; --"),
            Some("whatever"),
        ];

        for sample in samples {
            let label = normalize(sample);
            assert!(
                matches_database_grammar(&label),
                "{label:?} would be refused by the CHECK constraint"
            );
        }
    }

    /// Mirrors `user_signup_channel_grammar` from the migration.
    fn matches_database_grammar(label: &str) -> bool {
        if label == UNKNOWN || label == DIRECT {
            return true;
        }
        match label.split_once(':') {
            Some((family, suffix)) => {
                FAMILIES.contains(&family)
                    && (1..=MAX_SUFFIX_LEN).contains(&suffix.len())
                    && suffix.starts_with(|c: char| c.is_ascii_alphanumeric())
                    && suffix
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
            }
            None => false,
        }
    }
}

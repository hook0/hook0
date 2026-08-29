//! Every ceiling this generator works under, in one place so that raising one is a decision rather
//! than an accident.
//!
//! Each is a refusal, never a truncation. An example read up to a ceiling and no further would
//! produce a snippet ending mid-statement, which the dashboard would show to a user as if it were
//! whole — and showing something that is not what the SDK says is the failure this crate exists to
//! remove.

/// The most SDKs walked in one run, well above what ships today; a repository past this is one
/// where somebody should be raising this deliberately.
pub const MAX_SDKS: usize = 64;

/// The most bytes one example file is read under. The longest is under 3 KiB.
pub const MAX_EXAMPLE_BYTES: u64 = 128 * 1024;

/// The most bytes one manifest is read under.
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

/// The most bytes one extracted region may run to. A snippet longer than this is not something a
/// reader copies out of a dashboard panel.
pub const MAX_REGION_BYTES: usize = 16 * 1024;

/// The most characters a display name may run to.
pub const MAX_DISPLAY_NAME_CHARS: usize = 64;

/// The most characters the separator joining two labels may run to. It is punctuation and a line
/// break; anything longer is a template.
pub const MAX_SEPARATOR_CHARS: usize = 16;

/// The most characters either delimiter of a string literal may run to. Zig's `\\` line strings and
/// Python's triple quotes are the longest anything gets.
pub const MAX_DELIMITER_CHARS: usize = 16;

/// The most replacements one escaping rule may declare, and the most characters either half of one
/// may run to.
pub const MAX_ESCAPE_RULES: usize = 32;
pub const MAX_ESCAPE_CHARS: usize = 16;

/// The most characters the sentence behind a proof level may run to. It names a command and a job;
/// anything past this is a document, and belongs where documents go.
pub const MAX_PROVES_CHARS: usize = 512;

/// The most characters what a snippet needs beyond its own package may run to. It is a command or
/// two, or the handful of build-configuration lines a reader adds; anything past this is a tutorial,
/// and belongs where the client's README is.
pub const MAX_ALSO_NEEDS_CHARS: usize = 512;

/// The most lines of configuration one language may name as what puts its examples under its job,
/// and the most characters either those lines or the file holding them may run to. A naming is a
/// target declaration or a source root; a language needing more than this has hidden the answer
/// somewhere a reader cannot follow.
pub const MAX_REACH_LINES: usize = 8;
pub const MAX_REACH_CHARS: usize = 512;

/// The most characters the source written above a usage share may run to.
pub const MAX_SOURCE_CHARS: usize = 1024;

/// The bounds a usage share is held to, being a percentage of the respondents to one survey.
pub const MIN_USAGE_SHARE: f64 = 0.0;
pub const MAX_USAGE_SHARE: f64 = 100.0;

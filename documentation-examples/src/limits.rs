//! Every ceiling this checker works under, in one place so that raising one is a decision rather
//! than an accident.
//!
//! Each is a refusal, never a truncation: an example past a ceiling is reported and stops the run.
//! A checker that quietly stopped reading at the hundredth example would report success for the
//! ninety-nine it read, which is the failure mode this whole crate exists to remove.

use std::time::Duration;

/// The most pages read out of the documentation directory.
pub const MAX_PAGES: usize = 64;

/// The most bytes one page is read under. The longest SDK page is under 30 KiB.
pub const MAX_PAGE_BYTES: u64 = 512 * 1024;

/// The most examples taken from one page.
pub const MAX_EXAMPLES_PER_PAGE: usize = 64;

/// The most examples proven in one run, across every page.
pub const MAX_EXAMPLES: usize = 512;

/// The most bytes one example may be. A snippet longer than this is documentation nobody reads.
pub const MAX_EXAMPLE_BYTES: usize = 16 * 1024;

/// The most bytes one harness file is read under.
pub const MAX_HARNESS_BYTES: u64 = 128 * 1024;

/// The most regions one harness file may declare.
pub const MAX_REGIONS: usize = 32;

/// The most bytes one manifest is read under.
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

/// The most words one command may be spelled with.
pub const MAX_COMMAND_WORDS: usize = 32;

/// The most commands one language may declare, setup included.
pub const MAX_COMMANDS: usize = 8;

/// The most files one language's project scaffold may carry.
pub const MAX_SCAFFOLD_FILES: usize = 64;

/// The most bytes one scaffold file is read under.
pub const MAX_SCAFFOLD_BYTES: u64 = 1024 * 1024;

/// The most bytes of one command's output kept. Output past this is dropped from the *report*,
/// never from the verdict: the exit status decides, and the report says the output was cut.
pub const MAX_OUTPUT_BYTES: usize = 256 * 1024;

/// The longest a language may declare for one of its commands.
pub const MAX_TIMEOUT: Duration = Duration::from_secs(1800);

/// How often a running command is asked whether it is done.
pub const POLL_INTERVAL: Duration = Duration::from_millis(50);

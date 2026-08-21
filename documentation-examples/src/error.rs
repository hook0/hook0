//! Every way this checker refuses.
//!
//! There is no variant meaning "skipped". An example the checker cannot prove is an example that
//! failed, and every message below names the page and the language, because that is what the
//! person reading the pipeline needs in order to open the right file.

use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("could not read the directory {path}: {cause}")]
    ReadDirectory { path: String, cause: io::Error },

    #[error("could not read {path}: {cause}")]
    ReadFile { path: String, cause: io::Error },

    #[error("could not write {path}: {cause}")]
    WriteFile { path: String, cause: io::Error },

    #[error("{path} is {size} bytes, above the {maximum} one {what} is read under")]
    FileTooLarge {
        path: String,
        what: &'static str,
        size: u64,
        maximum: u64,
    },

    #[error("{path} holds {found} {what}, above the {maximum} allowed")]
    TooMany {
        path: String,
        what: &'static str,
        found: usize,
        maximum: usize,
    },

    #[error(
        "{page} carries no `sdkTarget` in its front matter, so nothing says which generated \
         client it documents; write `sdkTarget: <target>`, or `sdkTarget: none` for a page that \
         documents no single client"
    )]
    PageClaimsNothing { page: String },

    #[error(
        "{page} claims the target `{target}`, which the generator's registry does not produce; \
         the targets it does are {known}"
    )]
    PageClaimsUnknownTarget {
        page: String,
        target: String,
        known: String,
    },

    #[error("the target `{target}` is claimed by more than one page: {pages}")]
    TargetClaimedTwice { target: String, pages: String },

    #[error(
        "{page} claims the target `{target}` but carries no {target} example, so the page it \
         documents is proven by nothing; every page for a target must show at least one example \
         in that target's language"
    )]
    PageWithoutExample { page: String, target: String },

    #[error(
        "{page}:{line} opens a `{language}` example, and `{language}` is not a target the \
         generator's registry produces; the languages it does are {known}"
    )]
    ExampleInUnclaimedLanguage {
        page: String,
        line: usize,
        language: String,
        known: String,
    },

    #[error(
        "{page}:{line} is a `{language}` block with no `example=<region>` attribute, so nothing \
         says how it is assembled and it would go unproven; mark it with the harness region it \
         drops into, or move it to a language no target claims if it is prose"
    )]
    ExampleWithoutRegion {
        page: String,
        line: usize,
        language: String,
    },

    #[error("{page}:{line} carries the attribute `{attribute}`, which means nothing here")]
    UnknownAttribute {
        page: String,
        line: usize,
        attribute: String,
    },

    #[error("{page}:{line} is {size} bytes of {language}, above the {maximum} one example may be")]
    ExampleTooLarge {
        page: String,
        line: usize,
        language: String,
        size: usize,
        maximum: usize,
    },

    #[error("{page}:{line} opens a fence that is never closed")]
    UnclosedFence { page: String, line: usize },

    #[error(
        "the target `{target}` is documented by {page} but has no directory under {examples}, so \
         nothing says how a {target} example is assembled or what proving one means"
    )]
    TargetWithoutHarness {
        target: String,
        page: String,
        examples: String,
    },

    #[error(
        "{examples} holds a directory named `{directory}`, which no documented target answers to; \
         the targets documented here are {claimed}"
    )]
    HarnessWithoutTarget {
        examples: String,
        directory: String,
        claimed: String,
    },

    #[error("{path} is not a usable manifest: {cause}")]
    Manifest { path: String, cause: String },

    #[error("{path} declares no harness region; a region opens with `HARNESS <name>`")]
    HarnessWithoutRegion { path: String },

    #[error("{path} declares the region `{region}` twice")]
    RegionDeclaredTwice { path: String, region: String },

    #[error("{path} opens the region `{region}` and never closes it with `END HARNESS`")]
    RegionNotClosed { path: String, region: String },

    #[error("{path} closes a region that was never opened, at line {line}")]
    RegionNotOpened { path: String, line: usize },

    #[error(
        "{path} declares the region `{region}` with {found} `EXAMPLE` lines; a region takes \
         exactly one, which is where the snippet is written"
    )]
    RegionWithoutHole {
        path: String,
        region: String,
        found: usize,
    },

    #[error(
        "{page}:{line} asks for the harness region `{region}`, which {harness} does not declare; \
         it declares {known}"
    )]
    UnknownRegion {
        page: String,
        line: usize,
        region: String,
        harness: String,
        known: String,
    },

    #[error(
        "the {language} command `{command}` was still running after {seconds} s, the budget \
         {manifest} declares for it"
    )]
    Timeout {
        language: String,
        command: String,
        seconds: u64,
        manifest: String,
    },

    #[error("the {language} command `{command}` could not be started: {cause}")]
    CommandNotStarted {
        language: String,
        command: String,
        cause: io::Error,
    },

    #[error("{summary}")]
    ExamplesRefused { summary: String },
}

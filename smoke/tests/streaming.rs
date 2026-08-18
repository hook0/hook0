//! What a smoke says while it is saying it, and what of it the run keeps.
//!
//! Both halves matter and they used to be one: the harness inherited its children's streams, so a
//! failing smoke was readable as it happened and nothing was left for the run to read afterwards.
//! Every case here is a real child writing to a real pipe, since what is being asked is what
//! happens between two processes.

use std::path::Path;
use std::time::Duration;

use hook0_smoke::process::{Keep, stream};

/// Long enough for a `printf`, short enough that a case which hangs is a failure rather than a
/// suite nobody waits out.
const WITHIN: Duration = Duration::from_secs(30);

/// What the cases below pick out of a stream.
fn worth(line: &str) -> bool {
    line.starts_with("keep ")
}

/// Runs a shell one-liner, keeping at most `most` of the lines worth keeping.
fn said(script: &str, most: usize) -> Vec<String> {
    let ended = stream(
        "sh",
        &["-c".to_owned(), script.to_owned()],
        Path::new("."),
        &[],
        WITHIN,
        Keep { worth, most },
    )
    .expect("a shell runs");
    assert!(ended.ok, "the shell {}", ended.status);
    ended.kept
}

#[test]
fn the_lines_worth_keeping_are_kept_and_the_rest_are_only_written_through() {
    let kept = said("printf 'noise\\nkeep one\\nmore noise\\nkeep two\\n'", 16);

    assert_eq!(kept, vec!["keep one".to_owned(), "keep two".to_owned()]);
}

#[test]
fn a_line_a_child_never_ended_is_kept_all_the_same() {
    // What a report printed without a newline before the process exited arrives as. Dropping it
    // would turn one operation into one the run says was never driven.
    let kept = said("printf 'keep last'", 16);

    assert_eq!(kept, vec!["keep last".to_owned()]);
}

#[test]
fn what_a_child_says_on_its_error_stream_is_kept_too() {
    // Which stream a runtime prints on is the runtime's business: Java and .NET write plenty to
    // theirs. A protocol that only read one of them would hold half the languages to nothing.
    let kept = said("printf 'keep said\\n' >&2", 16);

    assert_eq!(kept, vec!["keep said".to_owned()]);
}

#[test]
fn a_carriage_return_before_the_newline_is_not_part_of_the_line() {
    let kept = said("printf 'keep one\\r\\n'", 16);

    assert_eq!(kept, vec!["keep one".to_owned()]);
}

#[test]
fn one_line_past_the_ceiling_is_kept_so_that_crossing_it_can_be_seen() {
    let kept = said("printf 'keep a\\nkeep b\\nkeep c\\nkeep d\\n'", 2);

    assert_eq!(kept.len(), 3, "one past the two allowed: {kept:?}");
}

#[test]
fn a_child_saying_more_than_a_pipe_holds_still_finishes() {
    // The property the threads are there for. A child whose output nobody drains stops when the
    // pipe buffer fills, and a deadline that only fires between polls would never be reached — so
    // this would hang rather than fail, which is the worst way for a harness to break.
    let kept = said(
        "for i in $(seq 1 20000); do printf 'a line of noise nobody keeps, number %s\\n' \"$i\"; \
         done; printf 'keep last\\n'",
        16,
    );

    assert_eq!(kept, vec!["keep last".to_owned()]);
}

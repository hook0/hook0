//! Reading out of an example the part of it a reader is shown.
//!
//! An example is a whole file so that a toolchain can compile it, and only part of a whole file is
//! worth showing. Two pairs of markers say which part: `hook0:snippet` delimits what is displayed,
//! and `hook0:label` delimits the one rendering of a label that the dashboard repeats.
//!
//! The markers are read as *tokens* rather than as lines, which is the property everything here
//! rests on. `rustfmt` moves `// hook0:label:end` up to the end of the line of code above it and
//! nothing talks it out of that; `gofmt` does the same. A reader that took whole lines would lose
//! that line of code, so a region is the text between two tokens and the comment introducing each
//! token is cut away by position.
//!
//! What survives is the indentation. A region is taken with the whitespace it sits on rather than
//! trimmed down to the code, because every repetition of it carries its own — which is also why the
//! separator joining two of them is punctuation and a line break and nothing else.

use crate::error::Error;
use crate::limits::MAX_REGION_BYTES;

/// The opening of what a reader is shown.
pub const SNIPPET_BEGIN: &str = "hook0:snippet:begin";
/// The close of it.
pub const SNIPPET_END: &str = "hook0:snippet:end";
/// The opening of the one rendering of a label.
pub const LABEL_BEGIN: &str = "hook0:label:begin";
/// The close of it.
pub const LABEL_END: &str = "hook0:label:end";

/// What every marker of this crate is spelled with, which is how one left inside an extracted
/// region is recognised without each of them being named again.
const MARKER_PREFIX: &str = "hook0:";

/// How much of a marker left inside a region a message quotes before it stops.
const MAX_REPORTED_MARKER_CHARS: usize = 48;

/// What a reader is shown, and nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub body: String,
}

/// What a reader is shown, and the one rendering of a label inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labelled {
    pub body: String,
    /// A substring of `body`, occurring exactly once: repeating a label is replacing it with as
    /// many copies of itself as the form carries, joined by the separator the manifest declares.
    pub label: String,
}

/// The snippet of an example that has no labels to repeat.
pub fn snippet(path: &str, text: &str, comment: &'static str) -> Result<Snippet, Error> {
    let shown = span(path, text, SNIPPET_BEGIN, SNIPPET_END, comment)?;
    let body = tidy(&text[shown.from..shown.to]);
    finish(path, SNIPPET_BEGIN, &body)?;
    Ok(Snippet { body })
}

/// The snippet of an example carrying a label, with the label's own markers taken out of it.
///
/// The label region is spliced back over the whole pair of label markers, which removes three
/// things in one move: the two marker comments, and the separator a formatter left at the end of
/// the region. What is left carries the label verbatim and exactly once, so repeating a label is a
/// string replacement — and repeating it no times leaves the container empty, which every one of
/// these languages accepts.
pub fn labelled(
    path: &str,
    text: &str,
    comment: &'static str,
    separator: &str,
) -> Result<Labelled, Error> {
    let shown = span(path, text, SNIPPET_BEGIN, SNIPPET_END, comment)?;
    let repeated = span(path, text, LABEL_BEGIN, LABEL_END, comment)?;
    if repeated.cut.start < shown.from || repeated.cut.end > shown.to {
        return Err(Error::LabelOutsideSnippet {
            path: path.to_owned(),
        });
    }

    let label = tidy(&strip_separator(
        &text[repeated.from..repeated.to],
        separator,
    ));
    finish(path, LABEL_BEGIN, &label)?;
    joins_cleanly(path, &label, separator)?;

    let raw = &text[shown.from..shown.to];
    let cut = (repeated.cut.start - shown.from)..(repeated.cut.end - shown.from);
    let body = tidy(&format!("{}{label}{}", &raw[..cut.start], &raw[cut.end..]));
    finish(path, SNIPPET_BEGIN, &body)?;

    let carried = body.match_indices(&label).count();
    if carried != 1 {
        return Err(Error::LabelNotOnceInSnippet {
            path: path.to_owned(),
            found: carried,
        });
    }

    Ok(Labelled { body, label })
}

/// What a pair of markers delimits: the region's own text, and the whole pair with its comments.
struct Span {
    /// Where the region's text starts, which is the line after the opening marker.
    from: usize,
    /// Where it stops, which is where the closing marker's comment opens.
    to: usize,
    /// The whole pair, comments included, which is what a caller splices over.
    cut: std::ops::Range<usize>,
}

/// Where a marker sits: the comment introducing it, and the token itself.
///
/// Two positions rather than one, because cutting a marker away means cutting its comment with it —
/// a `//` left behind on a line of code comments the rest of that line out.
struct Marker {
    /// Where a cut starts: the comment's opener, with the horizontal whitespace before it.
    cut_from: usize,
    token_end: usize,
}

/// Where a pair of markers puts its region, refusing every shape a reader could not trust.
fn span(
    path: &str,
    text: &str,
    begin: &'static str,
    end: &'static str,
    comment: &'static str,
) -> Result<Span, Error> {
    let opening = locate(path, text, begin, comment)?;
    let closing = locate(path, text, end, comment)?;
    if closing.cut_from < opening.token_end {
        return Err(Error::MarkersOutOfOrder {
            path: path.to_owned(),
            begin,
            end,
        });
    }
    nothing_follows(path, text, opening.token_end, begin)?;
    nothing_follows(path, text, closing.token_end, end)?;

    // A line comment runs to the end of its line, so nothing of the region can share the line the
    // opening marker is written on: the region starts on the next one.
    let from = match text[opening.token_end..].find('\n') {
        Some(newline) => opening.token_end + newline + 1,
        None => text.len(),
    };

    Ok(Span {
        from: from.min(closing.cut_from),
        to: closing.cut_from,
        cut: opening.cut_from..closing.token_end,
    })
}

/// The one occurrence of a marker, or a refusal saying how many there were instead.
fn locate(
    path: &str,
    text: &str,
    marker: &'static str,
    comment: &'static str,
) -> Result<Marker, Error> {
    let mut found = text.match_indices(marker);
    let (at, _) = match (found.next(), found.next()) {
        (Some(first), None) => first,
        (first, _) => {
            return Err(Error::MarkerNotOnce {
                path: path.to_owned(),
                marker,
                found: match first {
                    Some(_) => text.matches(marker).count(),
                    None => 0,
                },
            });
        }
    };

    // The comment introducing it has to be the last one opened before it, with nothing but
    // horizontal whitespace in between. A marker reached without one is a marker the language's own
    // toolchain reads as code.
    let opener = text[..at]
        .rfind(comment)
        .filter(|from| text[from + comment.len()..at].chars().all(is_horizontal))
        .ok_or(Error::MarkerNotInAComment {
            path: path.to_owned(),
            marker,
            comment,
        })?;

    Ok(Marker {
        cut_from: opener - trailing_horizontal(&text[..opener]),
        token_end: at + marker.len(),
    })
}

/// What follows a marker on its line, which has to be nothing.
///
/// Whatever shares a line with a marker is cut away with it, so a line carrying a marker and
/// something else would lose the something else without a word.
fn nothing_follows(path: &str, text: &str, at: usize, marker: &'static str) -> Result<(), Error> {
    let rest = &text[at..];
    let line = rest.split('\n').next().unwrap_or(rest);
    match line.trim().is_empty() {
        true => Ok(()),
        false => Err(Error::CodeBesideMarker {
            path: path.to_owned(),
            marker,
            trailing: line.trim().to_owned(),
        }),
    }
}

/// How many bytes of horizontal whitespace a text ends with.
fn trailing_horizontal(text: &str) -> usize {
    text.len() - text.trim_end_matches(is_horizontal).len()
}

/// A space or a tab: whitespace that is not the end of a line.
fn is_horizontal(character: char) -> bool {
    character == ' ' || character == '\t'
}

/// A region laid out for the separator that joins it.
///
/// A region carries the whitespace it sits on, because every repetition of it starts a line of its
/// own and that whitespace is what lays the line out. A separator with no line break in it breaks
/// that: the repetitions land on one line, and every one after the first drags an indent into the
/// middle of it. Java declared `", "` for a while, and three labels came out as one line with
/// twenty-two spaces between each pair — valid, and unreadable.
///
/// Refused rather than worked around. A second layout rule, applied only when a separator has no
/// line break, would be a rule with one user and one more thing to be wrong about; saying which two
/// declarations disagree, and how to settle it, costs nothing and cannot rot.
fn joins_cleanly(path: &str, label: &str, separator: &str) -> Result<(), Error> {
    let indent = label.len() - label.trim_start_matches(is_horizontal).len();
    match separator.contains(['\n', '\r']) || indent == 0 {
        true => Ok(()),
        false => Err(Error::IndentedRegionJoinedInline {
            path: path.to_owned(),
            separator: separator.to_owned(),
            indent,
        }),
    }
}

/// A region without the blank edges a marker's own line leaves behind.
///
/// Only the trailing edge loses everything. The leading edge loses line breaks alone, since the
/// indentation it opens on is what the region is laid out by.
fn tidy(text: &str) -> String {
    text.trim_start_matches(['\n', '\r']).trim_end().to_owned()
}

/// The region without the separator a formatter put back at the end of it.
///
/// `rustfmt` adds the trailing comma its language allows, and `Map.of("a", "b", "c", "d")` in Java
/// is a variadic argument list which allows none. Regions each carrying their own separator would
/// join into something valid in the first language and invalid in the second, so the separator
/// lives between repetitions and a region never carries one.
fn strip_separator(text: &str, separator: &str) -> String {
    let text = text.trim_end();
    let punctuation = separator.trim();
    match punctuation.is_empty() {
        true => text.to_owned(),
        false => text.strip_suffix(punctuation).unwrap_or(text).to_owned(),
    }
}

/// What has to be true of every region however it was read: it holds something, it is not longer
/// than a reader would copy, and no marker survived in it.
fn finish(path: &str, marker: &'static str, body: &str) -> Result<(), Error> {
    if body.is_empty() {
        return Err(Error::EmptyRegion {
            path: path.to_owned(),
            marker,
        });
    }
    if body.len() > MAX_REGION_BYTES {
        return Err(Error::RegionTooLarge {
            path: path.to_owned(),
            marker,
            size: body.len(),
            ceiling: MAX_REGION_BYTES,
        });
    }
    match surviving(body) {
        Some(left) => Err(Error::MarkerSurvivesExtraction {
            path: path.to_owned(),
            marker: left,
        }),
        None => Ok(()),
    }
}

/// A marker of this crate left inside an extracted region, if one is.
///
/// The prefix is only a marker when a word follows it, which is what tells `hook0:snippet:begin`
/// apart from the `hook0::` a language writes a path with. A misspelled marker is caught by the
/// same rule, which is the point of reading the prefix rather than the four names.
fn surviving(body: &str) -> Option<String> {
    body.match_indices(MARKER_PREFIX)
        .map(|(at, _)| &body[at..])
        .find(|left| {
            left[MARKER_PREFIX.len()..]
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic())
        })
        .map(|left| {
            left.split(char::is_whitespace)
                .next()
                .unwrap_or(left)
                .chars()
                .take(MAX_REPORTED_MARKER_CHARS)
                .collect()
        })
}

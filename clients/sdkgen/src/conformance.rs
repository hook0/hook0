//! The cross-cutting contract the hand-written half of every client follows, read as data.
//!
//! Retry classification, the bounds of a send and signature verification are written once per
//! language on purpose: network behaviour and cryptography do not templatise honestly. What that
//! costs is that a semantic is also *corrected* once per language, and that "this target reproduces
//! the reference behaviour" is checked by somebody reading one language and writing another.
//!
//! The corpus beside the targets is that contract as data instead: hand-authored, committed, and
//! read by the suite of every target. This module is what keeps it from drifting away from the API
//! it describes — the problems it classifies are held against the catalogue the snapshot declares,
//! and the statuses it rules on against the ones the selected operations answer. A problem the API
//! grows and nobody classifies stops here, rather than at the release of a client that quietly
//! decided for itself.
//!
//! Nothing here names a problem, a status or a header: those live in the corpus and in the
//! document, and this module only holds one against the other. Every input is bounded by
//! [`CorpusLimits`], and a corpus crossing a ceiling is refused with the ceiling it crossed rather
//! than trimmed down to fit.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;
use thiserror::Error as ThisError;

use crate::model::ErrorModel;

/// Member every document of the corpus opens with, saying what a change to it changes.
const HEADER_COMMENT: &str = "$comment";

/// The documents the corpus is made of. Each one has a shape of its own, so they are named here
/// rather than discovered: a document nobody reads is refused instead of sitting in the corpus
/// looking like contract.
const RETRY_DOCUMENT: &str = "retry.json";
const BOUNDS_DOCUMENT: &str = "bounds.json";
const SIGNATURE_DOCUMENT: &str = "signature.json";
const REQUEST_DOCUMENT: &str = "request.json";
const DOCUMENTS: [&str; 4] = [
    RETRY_DOCUMENT,
    BOUNDS_DOCUMENT,
    SIGNATURE_DOCUMENT,
    REQUEST_DOCUMENT,
];

/// Verdict a vector carrying a delivery that verifies is written under.
const ACCEPTED: &str = "accepted";

/// Verdict a vector carrying a delivery that is refused is written under.
const REFUSED: &str = "refused";

/// Lowest and highest status HTTP writes, which is all a rule may rule on.
const STATUSES: std::ops::RangeInclusive<u16> = 100..=599;

/// Ceilings applied to every corpus this module accepts.
///
/// Nothing is ever truncated: a corpus that crosses one of these is refused with the count it
/// reached and the ceiling it crossed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorpusLimits {
    /// Largest document accepted, in bytes, checked before any of it is parsed.
    pub max_document_bytes: u64,
    /// Largest number of entries the corpus directory may hold, walked before anything is read.
    pub max_documents: usize,
    /// Largest number of entries one list of the corpus may carry.
    pub max_entries: usize,
    /// Longest single string the corpus may carry, in bytes.
    pub max_text_bytes: usize,
}

impl CorpusLimits {
    /// Ceilings used when the caller has no reason to pick its own, each one an order of magnitude
    /// above what the corpus carries today.
    pub const DEFAULT: Self = Self {
        max_document_bytes: 512 * 1024,
        max_documents: 16,
        max_entries: 512,
        max_text_bytes: 4096,
    };
}

impl Default for CorpusLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// What a client does with a failure that produced no answer to read.
///
/// This is what the type carrying the failure cannot say: a reset connection, a response above the
/// client's own ceiling and a URL nothing can be sent to all arrive as one type in most runtimes,
/// and only one of them could end differently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportRule {
    pub cause: String,
    pub retryable: bool,
    pub reason: String,
}

/// What a client does with an answer whose status is all it has to go on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusRule {
    pub status: u16,
    pub retryable: bool,
    pub reason: String,
}

/// What a client does with an answer naming one problem of the API's catalogue.
///
/// This is what a status alone cannot say: two problems answering under the same status can call
/// for opposite behaviour, and the identifier is what tells them apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemRule {
    pub problem: String,
    pub status: u16,
    pub retryable: bool,
    pub reason: String,
}

/// One value of the delay header, and what a client is expected to make of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelayCase {
    pub name: String,
    /// The header value as it arrives, whether or not it reads as a delay.
    pub header: String,
    /// How many seconds it asks for, absent when it asks for nothing a client can read.
    pub honoured: Option<u64>,
}

/// The delay the API names beside a retryable answer, and the values a client has to survive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryAfter {
    pub header: String,
    pub cases: Vec<DelayCase>,
}

/// The bounds one send is held to, in milliseconds and in bytes.
///
/// The last three bound what the other end may cost the caller rather than what the caller does:
/// a client with no ceiling on what it reads has no answer to a server that never stops writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    pub max_attempts: u64,
    pub max_attempts_cap: u64,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub max_total_delay_ms: u64,
    pub request_timeout_ms: u64,
    pub max_payload_bytes: u64,
    pub max_response_bytes: u64,
    pub max_response_headers: u64,
    pub max_header_bytes: u64,
}

/// One header every request carries, and which requests carry it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderRule {
    pub name: String,
    pub value: String,
    /// Which requests carry it, named among the occasions the document declares.
    pub when: String,
    pub reason: String,
}

/// What every client puts on the wire beside the body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestFormat {
    /// The occasions a header may be carried on, as the corpus declares them.
    pub occasions: Vec<String>,
    pub headers: Vec<HeaderRule>,
}

/// One delivery, and whether verifying it is expected to accept or refuse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vector {
    pub name: String,
    pub secret: String,
    pub payload: String,
    /// Headers as the request carried them, in the order it carried them.
    pub headers: Vec<(String, String)>,
    pub signature: String,
    /// Moment the signature is held against, in whole seconds since the epoch.
    pub current_time: i64,
    pub tolerance_seconds: i64,
    /// Which refusal the delivery is expected to meet, absent when it verifies.
    pub refusal: Option<String>,
    pub reason: String,
}

impl Vector {
    /// Whether verifying this delivery is expected to accept it.
    pub fn verifies(&self) -> bool {
        self.refusal.is_none()
    }
}

/// The whole contract, as the committed corpus writes it down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Corpus {
    pub transport: Vec<TransportRule>,
    pub statuses: Vec<StatusRule>,
    pub problems: Vec<ProblemRule>,
    pub retry_after: RetryAfter,
    pub bounds: Bounds,
    /// The names a refused vector may be refused under, as the corpus declares them.
    pub refusals: Vec<String>,
    pub vectors: Vec<Vector>,
    pub request: RequestFormat,
}

impl Corpus {
    /// Reads the corpus out of a directory, refusing anything it cannot read whole.
    pub fn read(directory: &Path, limits: &CorpusLimits) -> Result<Self, ConformanceError> {
        let present = documents_under(directory, limits)?;
        for expected in DOCUMENTS {
            if !present.contains(expected) {
                return Err(ConformanceError::MissingDocument {
                    document: expected.to_owned(),
                });
            }
        }

        let retry = document(directory, RETRY_DOCUMENT, limits)?;
        let bounds = document(directory, BOUNDS_DOCUMENT, limits)?;
        let signature = document(directory, SIGNATURE_DOCUMENT, limits)?;
        let request = document(directory, REQUEST_DOCUMENT, limits)?;

        let corpus = Self {
            transport: transport_rules(RETRY_DOCUMENT, &retry, limits)?,
            statuses: status_rules(RETRY_DOCUMENT, &retry, limits)?,
            problems: problem_rules(RETRY_DOCUMENT, &retry, limits)?,
            retry_after: retry_after(RETRY_DOCUMENT, &retry, limits)?,
            bounds: bounds_of(BOUNDS_DOCUMENT, &bounds)?,
            refusals: refusals(SIGNATURE_DOCUMENT, &signature, limits)?,
            vectors: vectors(SIGNATURE_DOCUMENT, &signature, limits)?,
            request: request_format(REQUEST_DOCUMENT, &request, limits)?,
        };
        corpus.is_consistent()?;

        Ok(corpus)
    }

    /// Holds the corpus against the error contract the snapshot describes.
    ///
    /// Both directions are checked. A problem the corpus classifies and the API does not declare is
    /// a rename or a typo nobody would otherwise notice, since a verdict for a problem that never
    /// arrives is never exercised. A problem the API declares and the corpus does not classify is
    /// the one that matters: whether a client repeats a request that met it is a decision somebody
    /// has to make, and every target would otherwise make it on its own.
    pub fn agrees_with(&self, errors: &ErrorModel) -> Result<(), ConformanceError> {
        let declared: BTreeSet<&str> = errors
            .catalogue
            .values()
            .iter()
            .map(String::as_str)
            .collect();
        let classified: BTreeSet<&str> = self
            .problems
            .iter()
            .map(|rule| rule.problem.as_str())
            .collect();

        if let Some(unknown) = classified.difference(&declared).next() {
            return Err(ConformanceError::UnknownProblem {
                problem: (*unknown).to_owned(),
            });
        }
        if let Some(unclassified) = declared.difference(&classified).next() {
            return Err(ConformanceError::UnclassifiedProblem {
                problem: (*unclassified).to_owned(),
            });
        }

        let answered: BTreeSet<u16> = errors.statuses.iter().copied().collect();
        let ruled: BTreeSet<u16> = self.statuses.iter().map(|rule| rule.status).collect();

        if let Some(status) = ruled.difference(&answered).next() {
            return Err(ConformanceError::UnansweredStatus { status: *status });
        }
        if let Some(status) = answered.difference(&ruled).next() {
            return Err(ConformanceError::UnruledStatus { status: *status });
        }
        for rule in &self.problems {
            if !answered.contains(&rule.status) {
                return Err(ConformanceError::UnansweredStatus {
                    status: rule.status,
                });
            }
        }

        Ok(())
    }

    /// What the corpus holds true on its own, whatever API it is held against.
    fn is_consistent(&self) -> Result<(), ConformanceError> {
        let bounds = &self.bounds;
        if bounds.max_attempts == 0 {
            return Err(ConformanceError::UnusableBounds {
                reason: "a send making no attempt sends nothing".to_owned(),
            });
        }
        if bounds.max_attempts > bounds.max_attempts_cap {
            return Err(ConformanceError::UnusableBounds {
                reason: "the attempts made by default sit above the cap nothing may cross"
                    .to_owned(),
            });
        }
        if bounds.initial_backoff_ms > bounds.max_backoff_ms {
            return Err(ConformanceError::UnusableBounds {
                reason: "the first delay sits above the ceiling no delay may exceed".to_owned(),
            });
        }
        if bounds.max_backoff_ms > bounds.max_total_delay_ms {
            return Err(ConformanceError::UnusableBounds {
                reason: "one delay may spend more than the budget every delay shares".to_owned(),
            });
        }
        if bounds.request_timeout_ms == 0 || bounds.max_payload_bytes == 0 {
            return Err(ConformanceError::UnusableBounds {
                reason: "an attempt is given no time, or a payload no room".to_owned(),
            });
        }
        if bounds.max_response_bytes == 0
            || bounds.max_response_headers == 0
            || bounds.max_header_bytes == 0
        {
            return Err(ConformanceError::UnusableBounds {
                reason: "a ceiling on what the other end may send is zero, which reads nothing at all rather than bounding what is read".to_owned(),
            });
        }

        let declared: BTreeSet<&str> = self.refusals.iter().map(String::as_str).collect();
        let exercised: BTreeSet<&str> = self
            .vectors
            .iter()
            .filter_map(|vector| vector.refusal.as_deref())
            .collect();

        if let Some(unknown) = exercised.difference(&declared).next() {
            let vector = self
                .vectors
                .iter()
                .find(|vector| vector.refusal.as_deref() == Some(*unknown))
                .map(|vector| vector.name.clone())
                .unwrap_or_default();
            return Err(ConformanceError::UnknownRefusal {
                vector,
                refusal: (*unknown).to_owned(),
            });
        }

        if !self.vectors.iter().any(Vector::verifies) {
            return Err(ConformanceError::NothingVerifies);
        }

        if let Some(unexercised) = declared.difference(&exercised).next() {
            return Err(ConformanceError::UnexercisedRefusal {
                refusal: (*unexercised).to_owned(),
            });
        }

        let occasions: BTreeSet<&str> = self.request.occasions.iter().map(String::as_str).collect();
        let carried: BTreeSet<&str> = self
            .request
            .headers
            .iter()
            .map(|header| header.when.as_str())
            .collect();

        if let Some(unknown) = carried.difference(&occasions).next() {
            let header = self
                .request
                .headers
                .iter()
                .find(|header| header.when == *unknown)
                .map(|header| header.name.clone())
                .unwrap_or_default();
            return Err(ConformanceError::UnknownOccasion {
                header,
                occasion: (*unknown).to_owned(),
            });
        }
        if let Some(unexercised) = occasions.difference(&carried).next() {
            return Err(ConformanceError::UnexercisedOccasion {
                occasion: (*unexercised).to_owned(),
            });
        }

        Ok(())
    }
}

/// Everything that stops a corpus from being read, or from describing the API it sits beside.
#[derive(Debug, Clone, PartialEq, Eq, ThisError)]
pub enum ConformanceError {
    #[error("the corpus at {directory} could not be walked: {reason}")]
    DirectoryUnreadable { directory: String, reason: String },

    #[error("`{document}` of the corpus could not be read: {reason}")]
    DocumentUnreadable { document: String, reason: String },

    #[error("`{document}` is {size} bytes long, above the {limit} accepted")]
    DocumentTooLarge {
        document: String,
        size: u64,
        limit: u64,
    },

    #[error("the corpus holds {count} entries, above the {limit} accepted")]
    TooManyDocuments { count: usize, limit: usize },

    #[error("the corpus holds `{document}`, which nothing reads and nothing checks")]
    UnreadDocument { document: String },

    #[error("the corpus holds no `{document}`")]
    MissingDocument { document: String },

    #[error("`{document}` is not a usable document: {reason}")]
    Malformed { document: String, reason: String },

    #[error(
        "`{document}` opens with no `{HEADER_COMMENT}` saying that it is shared and what a change to it changes"
    )]
    Uncommented { document: String },

    #[error("`{document}` declares no `{subject}`")]
    Missing { document: String, subject: String },

    #[error("`{subject}` of `{document}` is not {expected}")]
    Unreadable {
        document: String,
        subject: String,
        expected: String,
    },

    #[error("`{subject}` of `{document}` is {size} bytes long, above the {limit} accepted")]
    TextTooLong {
        document: String,
        subject: String,
        size: usize,
        limit: usize,
    },

    #[error("`{subject}` of `{document}` carries {count} entries, above the {limit} accepted")]
    TooManyEntries {
        document: String,
        subject: String,
        count: usize,
        limit: usize,
    },

    #[error("`{subject}` of `{document}` carries nothing at all")]
    Empty { document: String, subject: String },

    #[error("`{subject}` of `{document}` carries `{entry}` twice, so one of the two says nothing")]
    Duplicated {
        document: String,
        subject: String,
        entry: String,
    },

    #[error("the corpus classifies `{problem}`, which the API's problem catalogue does not carry")]
    UnknownProblem { problem: String },

    #[error(
        "the API's problem catalogue carries `{problem}`, which the corpus does not classify, so nothing says whether a client repeats a request that met it"
    )]
    UnclassifiedProblem { problem: String },

    #[error("the corpus rules on a status of {status}, which no selected operation answers")]
    UnansweredStatus { status: u16 },

    #[error("the API answers a status of {status}, which the corpus does not rule on")]
    UnruledStatus { status: u16 },

    #[error("the bounds of the corpus are not usable: {reason}")]
    UnusableBounds { reason: String },

    #[error(
        "vector `{vector}` is refused as `{refusal}`, which the corpus declares no refusal for"
    )]
    UnknownRefusal { vector: String, refusal: String },

    #[error(
        "no vector of the corpus verifies, so nothing says what an accepted delivery looks like"
    )]
    NothingVerifies,

    #[error(
        "the corpus declares the refusal `{refusal}` and carries no vector meeting it, so every target would have to map a name none of them ever answers"
    )]
    UnexercisedRefusal { refusal: String },

    #[error(
        "the `{header}` header is carried on `{occasion}`, which is not one of the occasions the corpus declares"
    )]
    UnknownOccasion { header: String, occasion: String },

    #[error(
        "the corpus declares the occasion `{occasion}` and puts no header on it, so nothing says what a request made then carries"
    )]
    UnexercisedOccasion { occasion: String },
}

/// The entries the corpus directory holds, refusing one nothing reads.
fn documents_under(
    directory: &Path,
    limits: &CorpusLimits,
) -> Result<BTreeSet<String>, ConformanceError> {
    let unwalkable = |reason: String| ConformanceError::DirectoryUnreadable {
        directory: directory.display().to_string(),
        reason,
    };

    let entries = fs::read_dir(directory).map_err(|err| unwalkable(err.to_string()))?;

    let mut present = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|err| unwalkable(err.to_string()))?;
        if present.len() == limits.max_documents {
            return Err(ConformanceError::TooManyDocuments {
                count: present.len() + 1,
                limit: limits.max_documents,
            });
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        if !DOCUMENTS.contains(&name.as_str()) {
            return Err(ConformanceError::UnreadDocument { document: name });
        }
        present.insert(name);
    }

    Ok(present)
}

/// One document of the corpus, bounded before it is parsed and refused unless it says what it is.
fn document(
    directory: &Path,
    document: &str,
    limits: &CorpusLimits,
) -> Result<Value, ConformanceError> {
    let path = directory.join(document);
    let unreadable = |reason: String| ConformanceError::DocumentUnreadable {
        document: document.to_owned(),
        reason,
    };

    let metadata = fs::metadata(&path).map_err(|err| unreadable(err.to_string()))?;
    if metadata.len() > limits.max_document_bytes {
        return Err(ConformanceError::DocumentTooLarge {
            document: document.to_owned(),
            size: metadata.len(),
            limit: limits.max_document_bytes,
        });
    }

    let bytes = fs::read(&path).map_err(|err| unreadable(err.to_string()))?;
    let value: Value =
        serde_json::from_slice(&bytes).map_err(|err| ConformanceError::Malformed {
            document: document.to_owned(),
            reason: err.to_string(),
        })?;
    if !value.is_object() {
        return Err(ConformanceError::Malformed {
            document: document.to_owned(),
            reason: "it is not a JSON object".to_owned(),
        });
    }

    if !is_commented(&value) {
        return Err(ConformanceError::Uncommented {
            document: document.to_owned(),
        });
    }

    Ok(value)
}

/// Whether a document opens with the lines saying what it is and what changing it changes.
fn is_commented(value: &Value) -> bool {
    value
        .get(HEADER_COMMENT)
        .and_then(Value::as_array)
        .is_some_and(|lines| {
            !lines.is_empty()
                && lines
                    .iter()
                    .all(|line| line.as_str().is_some_and(|line| !line.trim().is_empty()))
        })
}

/// One member of a document, which it is not the document the corpus needs without.
fn member<'a>(
    document: &str,
    holder: &'a Value,
    subject: &str,
) -> Result<&'a Value, ConformanceError> {
    holder
        .get(subject)
        .ok_or_else(|| ConformanceError::Missing {
            document: document.to_owned(),
            subject: subject.to_owned(),
        })
}

/// One string of a document, bounded by what the corpus may carry.
fn text(
    document: &str,
    holder: &Value,
    subject: &str,
    limits: &CorpusLimits,
) -> Result<String, ConformanceError> {
    let read = holder.get(subject).and_then(Value::as_str).ok_or_else(|| {
        ConformanceError::Unreadable {
            document: document.to_owned(),
            subject: subject.to_owned(),
            expected: "text".to_owned(),
        }
    })?;

    if read.len() > limits.max_text_bytes {
        return Err(ConformanceError::TextTooLong {
            document: document.to_owned(),
            subject: subject.to_owned(),
            size: read.len(),
            limit: limits.max_text_bytes,
        });
    }
    if read.trim().is_empty() {
        return Err(ConformanceError::Empty {
            document: document.to_owned(),
            subject: subject.to_owned(),
        });
    }

    Ok(read.to_owned())
}

/// One flag of a document.
fn flag(document: &str, holder: &Value, subject: &str) -> Result<bool, ConformanceError> {
    holder
        .get(subject)
        .and_then(Value::as_bool)
        .ok_or_else(|| ConformanceError::Unreadable {
            document: document.to_owned(),
            subject: subject.to_owned(),
            expected: "a flag".to_owned(),
        })
}

/// One whole number of a document, nothing the corpus counts ever being negative.
fn count(document: &str, holder: &Value, subject: &str) -> Result<u64, ConformanceError> {
    holder
        .get(subject)
        .and_then(Value::as_u64)
        .ok_or_else(|| ConformanceError::Unreadable {
            document: document.to_owned(),
            subject: subject.to_owned(),
            expected: "a whole number".to_owned(),
        })
}

/// One moment or span of a document, which may sit either side of zero.
fn seconds(document: &str, holder: &Value, subject: &str) -> Result<i64, ConformanceError> {
    holder
        .get(subject)
        .and_then(Value::as_i64)
        .ok_or_else(|| ConformanceError::Unreadable {
            document: document.to_owned(),
            subject: subject.to_owned(),
            expected: "a whole number of seconds".to_owned(),
        })
}

/// One status of a document, refused unless it is one HTTP writes.
fn status(document: &str, holder: &Value, subject: &str) -> Result<u16, ConformanceError> {
    u16::try_from(count(document, holder, subject)?)
        .ok()
        .filter(|status| STATUSES.contains(status))
        .ok_or_else(|| ConformanceError::Unreadable {
            document: document.to_owned(),
            subject: subject.to_owned(),
            expected: "a status".to_owned(),
        })
}

/// One list of a document, bounded by what the corpus may carry and never empty.
fn entries<'a>(
    document: &str,
    holder: &'a Value,
    subject: &str,
    limits: &CorpusLimits,
) -> Result<&'a Vec<Value>, ConformanceError> {
    let read = member(document, holder, subject)?
        .as_array()
        .ok_or_else(|| ConformanceError::Unreadable {
            document: document.to_owned(),
            subject: subject.to_owned(),
            expected: "a list".to_owned(),
        })?;

    if read.len() > limits.max_entries {
        return Err(ConformanceError::TooManyEntries {
            document: document.to_owned(),
            subject: subject.to_owned(),
            count: read.len(),
            limit: limits.max_entries,
        });
    }
    if read.is_empty() {
        return Err(ConformanceError::Empty {
            document: document.to_owned(),
            subject: subject.to_owned(),
        });
    }

    Ok(read)
}

/// Refuses a list naming the same thing twice, which would leave one of the two unread.
fn distinct(
    document: &str,
    subject: &str,
    named: impl Iterator<Item = String>,
) -> Result<(), ConformanceError> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for name in named {
        if !seen.insert(name.clone()) {
            return Err(ConformanceError::Duplicated {
                document: document.to_owned(),
                subject: subject.to_owned(),
                entry: name,
            });
        }
    }
    Ok(())
}

fn transport_rules(
    document: &str,
    holder: &Value,
    limits: &CorpusLimits,
) -> Result<Vec<TransportRule>, ConformanceError> {
    let read = member(document, holder, "transport")?;

    let mut rules = Vec::new();
    for entry in entries(document, read, "causes", limits)? {
        rules.push(TransportRule {
            cause: text(document, entry, "cause", limits)?,
            retryable: flag(document, entry, "retryable")?,
            reason: text(document, entry, "reason", limits)?,
        });
    }

    distinct(
        document,
        "causes",
        rules.iter().map(|rule| rule.cause.clone()),
    )?;
    Ok(rules)
}

fn request_format(
    document: &str,
    holder: &Value,
    limits: &CorpusLimits,
) -> Result<RequestFormat, ConformanceError> {
    let mut occasions = Vec::new();
    for entry in entries(document, holder, "occasions", limits)? {
        occasions.push(name_of(document, entry, "occasions", limits)?);
    }
    distinct(document, "occasions", occasions.iter().cloned())?;

    let mut headers = Vec::new();
    for entry in entries(document, holder, "headers", limits)? {
        headers.push(HeaderRule {
            name: text(document, entry, "name", limits)?,
            value: text(document, entry, "value", limits)?,
            when: text(document, entry, "when", limits)?,
            reason: text(document, entry, "reason", limits)?,
        });
    }

    // HTTP compares header names without regard to case, so two rules differing only in case are
    // two rules for one header, and which of them a target honoured would be its own business.
    distinct(
        document,
        "headers",
        headers.iter().map(|header| header.name.to_lowercase()),
    )?;

    Ok(RequestFormat { occasions, headers })
}

/// One name of a list of names, bounded by what the corpus may carry.
fn name_of(
    document: &str,
    entry: &Value,
    subject: &str,
    limits: &CorpusLimits,
) -> Result<String, ConformanceError> {
    let read = entry.as_str().ok_or_else(|| ConformanceError::Unreadable {
        document: document.to_owned(),
        subject: subject.to_owned(),
        expected: "a list of names".to_owned(),
    })?;
    if read.len() > limits.max_text_bytes {
        return Err(ConformanceError::TextTooLong {
            document: document.to_owned(),
            subject: subject.to_owned(),
            size: read.len(),
            limit: limits.max_text_bytes,
        });
    }
    if read.trim().is_empty() {
        return Err(ConformanceError::Empty {
            document: document.to_owned(),
            subject: subject.to_owned(),
        });
    }
    Ok(read.to_owned())
}

fn status_rules(
    document: &str,
    holder: &Value,
    limits: &CorpusLimits,
) -> Result<Vec<StatusRule>, ConformanceError> {
    let mut rules = Vec::new();
    for entry in entries(document, holder, "statuses", limits)? {
        rules.push(StatusRule {
            status: status(document, entry, "status")?,
            retryable: flag(document, entry, "retryable")?,
            reason: text(document, entry, "reason", limits)?,
        });
    }

    distinct(
        document,
        "statuses",
        rules.iter().map(|rule| rule.status.to_string()),
    )?;
    Ok(rules)
}

fn problem_rules(
    document: &str,
    holder: &Value,
    limits: &CorpusLimits,
) -> Result<Vec<ProblemRule>, ConformanceError> {
    let mut rules = Vec::new();
    for entry in entries(document, holder, "problems", limits)? {
        rules.push(ProblemRule {
            problem: text(document, entry, "problem", limits)?,
            status: status(document, entry, "status")?,
            retryable: flag(document, entry, "retryable")?,
            reason: text(document, entry, "reason", limits)?,
        });
    }

    distinct(
        document,
        "problems",
        rules.iter().map(|rule| rule.problem.clone()),
    )?;
    Ok(rules)
}

fn retry_after(
    document: &str,
    holder: &Value,
    limits: &CorpusLimits,
) -> Result<RetryAfter, ConformanceError> {
    let read = member(document, holder, "retry_after")?;

    let mut cases = Vec::new();
    for entry in entries(document, read, "cases", limits)? {
        let honoured = match flag(document, entry, "honoured")? {
            true => Some(count(document, entry, "seconds")?),
            false => None,
        };
        cases.push(DelayCase {
            name: text(document, entry, "name", limits)?,
            header: text(document, entry, "header", limits)?,
            honoured,
        });
    }

    distinct(
        document,
        "cases",
        cases.iter().map(|case| case.name.clone()),
    )?;

    Ok(RetryAfter {
        header: text(document, read, "header", limits)?,
        cases,
    })
}

fn bounds_of(document: &str, holder: &Value) -> Result<Bounds, ConformanceError> {
    let read = member(document, holder, "bounds")?;
    Ok(Bounds {
        max_attempts: count(document, read, "max_attempts")?,
        max_attempts_cap: count(document, read, "max_attempts_cap")?,
        initial_backoff_ms: count(document, read, "initial_backoff_ms")?,
        max_backoff_ms: count(document, read, "max_backoff_ms")?,
        max_total_delay_ms: count(document, read, "max_total_delay_ms")?,
        request_timeout_ms: count(document, read, "request_timeout_ms")?,
        max_payload_bytes: count(document, read, "max_payload_bytes")?,
        max_response_bytes: count(document, read, "max_response_bytes")?,
        max_response_headers: count(document, read, "max_response_headers")?,
        max_header_bytes: count(document, read, "max_header_bytes")?,
    })
}

fn refusals(
    document: &str,
    holder: &Value,
    limits: &CorpusLimits,
) -> Result<Vec<String>, ConformanceError> {
    let mut declared = Vec::new();
    for entry in entries(document, holder, "refusals", limits)? {
        declared.push(name_of(document, entry, "refusals", limits)?);
    }

    distinct(document, "refusals", declared.iter().cloned())?;
    Ok(declared)
}

fn vectors(
    document: &str,
    holder: &Value,
    limits: &CorpusLimits,
) -> Result<Vec<Vector>, ConformanceError> {
    let mut read = Vec::new();
    for entry in entries(document, holder, "vectors", limits)? {
        let name = text(document, entry, "name", limits)?;
        let verdict = text(document, entry, "verdict", limits)?;

        let refusal = match verdict.as_str() {
            ACCEPTED => None,
            REFUSED => Some(text(document, entry, "refusal", limits)?),
            _ => {
                return Err(ConformanceError::Unreadable {
                    document: document.to_owned(),
                    subject: format!("verdict of `{name}`"),
                    expected: format!("`{ACCEPTED}` or `{REFUSED}`"),
                });
            }
        };

        read.push(Vector {
            secret: text(document, entry, "secret", limits)?,
            payload: text(document, entry, "payload", limits)?,
            headers: headers(document, entry, &name, limits)?,
            signature: text(document, entry, "signature", limits)?,
            current_time: seconds(document, entry, "current_time")?,
            tolerance_seconds: tolerance(document, entry, &name)?,
            reason: text(document, entry, "reason", limits)?,
            refusal,
            name,
        });
    }

    distinct(
        document,
        "vectors",
        read.iter().map(|vector| vector.name.clone()),
    )?;
    Ok(read)
}

/// The headers one vector says the request carried, in the order it carried them.
fn headers(
    document: &str,
    entry: &Value,
    vector: &str,
    limits: &CorpusLimits,
) -> Result<Vec<(String, String)>, ConformanceError> {
    let unreadable = || ConformanceError::Unreadable {
        document: document.to_owned(),
        subject: format!("headers of `{vector}`"),
        expected: "a list of name and value pairs".to_owned(),
    };

    let carried = entry
        .get("headers")
        .and_then(Value::as_array)
        .ok_or_else(unreadable)?;
    if carried.len() > limits.max_entries {
        return Err(ConformanceError::TooManyEntries {
            document: document.to_owned(),
            subject: format!("headers of `{vector}`"),
            count: carried.len(),
            limit: limits.max_entries,
        });
    }

    let mut read = Vec::with_capacity(carried.len());
    for pair in carried {
        let [name, value] = pair.as_array().map(Vec::as_slice).unwrap_or_default() else {
            return Err(unreadable());
        };
        let (Some(name), Some(value)) = (name.as_str(), value.as_str()) else {
            return Err(unreadable());
        };
        if name.len() > limits.max_text_bytes || value.len() > limits.max_text_bytes {
            return Err(ConformanceError::TextTooLong {
                document: document.to_owned(),
                subject: format!("headers of `{vector}`"),
                size: name.len().max(value.len()),
                limit: limits.max_text_bytes,
            });
        }
        read.push((name.to_owned(), value.to_owned()));
    }

    Ok(read)
}

/// The window one vector holds its moment within, which is no window at all once it reaches zero.
fn tolerance(document: &str, entry: &Value, vector: &str) -> Result<i64, ConformanceError> {
    let read = seconds(document, entry, "tolerance_seconds")?;
    if read <= 0 {
        return Err(ConformanceError::Unreadable {
            document: document.to_owned(),
            subject: format!("tolerance_seconds of `{vector}`"),
            expected: "a window wider than nothing".to_owned(),
        });
    }
    Ok(read)
}

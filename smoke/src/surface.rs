//! The one line a smoke prints per operation it drove, and the bijection every language is held to.
//!
//! Each client's loopback suite drives its whole generated surface, and every one of them drives it
//! against a server the suite itself wrote. That server encodes the test author's belief about how
//! Hook0 answers, and nothing checks the belief: a model whose field names do not match what the
//! API really returns passes twelve suites and fails on a consumer's first call. Only a real
//! instance can settle that, and only the client's own generated layer can be the thing that reads
//! the answer.
//!
//! So a smoke says, on its own output, one line per operation it drove:
//!
//! ```text
//! exercised <operationId> accepted
//! exercised <operationId> refused:<problemId>
//! ```
//!
//! `accepted` is the API answering a success and the client decoding it into the generated value.
//! `refused:<problemId>` is the API answering a problem document and the client decoding it as that
//! problem. Both are a complete round trip through the generated layer against a real Hook0, and
//! both count — an operation the smoke's credential may not perform still proves the client
//! composed the request, reached the API and read what came back, which is the whole question.
//! Anything else — a transport failure, an answer the client cannot decode, a problem id it does not
//! know — is the smoke's own failure to report, and it exits non-zero saying so.
//!
//! One line per generated model type it decoded, too:
//!
//! ```text
//! decoded <ModelName>
//! ```
//!
//! Operations alone are not enough, and the gap is not small. Every one of them could come back
//! refused — an unauthorised credential refuses in exactly the shape an authorised one accepts —
//! and a client that can decode nothing at all would satisfy the operation bijection while never
//! having parsed a single generated model out of a real answer. That is the bug class this exists
//! to catch. So the models are held to a bijection of their own, against the types the generator
//! emits that some operation really answers.
//!
//! What the set of lines is held against is not written here. It is the operations the API document
//! declares under the tag that target's client is generated from — [`hook0_sdkgen::PUBLIC_TAG`] for
//! the SDKs, and whatever else the registry says for a target that selects differently — read
//! through the generator itself, so an operation the API grows fails every language until somebody
//! drives it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use hook0_sdkgen::targets::Decoding;
use hook0_sdkgen::{ApiModel, Limits, ObjectShape, Shape, Snapshot};

use crate::error::Error;

/// The word an operation's report opens with.
pub const PREFIX: &str = "exercised";

/// The word a model's report opens with.
pub const DECODED: &str = "decoded";

/// What a smoke says when the API answered a success and the client read the generated value out
/// of it.
pub const ACCEPTED: &str = "accepted";

/// What a smoke opens the other outcome with, the problem id the client read following it.
pub const REFUSED: &str = "refused:";

/// The one problem a report may not name.
///
/// Hook0 paces callers per credential, and a flow driving three dozen operations one after another
/// is what that pacing is for. A refusal is otherwise as good as a success here — it is still a
/// round trip the client composed and read — but this one is not about the operation at all: it
/// says the instance never looked. A language that let it through would report every operation as
/// driven while proving nothing about any of them, which is the one way this whole exercise can
/// pass and mean nothing. So a smoke waits out the delay the answer names and asks again; the name
/// is here because that is the only way the run can tell it did not.
pub const THROTTLED: &str = "RateLimited";

/// The most reports one smoke may make before the run stops reading them.
///
/// The API declares a few dozen operations and a flow may legitimately drive one of them more than
/// once, so this sits well above what any language needs. What it bounds is a smoke printing the
/// word in a loop: the lines are kept in memory to be counted, and a stream nobody bounds is one
/// that eventually costs the machine rather than the run.
pub const MAX_REPORTS: usize = 512;

/// What one operation did against the real instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The API answered a success and the client decoded it.
    Accepted,
    /// The API answered a problem document and the client decoded it as the problem named.
    Refused(String),
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accepted => f.write_str(ACCEPTED),
            Self::Refused(problem) => write!(f, "{REFUSED}{problem}"),
        }
    }
}

/// One line a smoke printed, once it has been read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Report {
    /// An operation driven against the instance, and how the instance answered it.
    Exercised(String, Outcome),
    /// A generated model type this client decoded out of a real answer.
    Decoded(String),
}

/// Whether a line a smoke printed is a report, which is what decides if it is kept as one.
///
/// Deliberately loose about what follows the word: a line is a report because it opens with one of
/// the two, and everything else about it is then held to the grammar by [`report`]. A line that
/// opens with the word and is malformed is a failure rather than a line quietly passed over, which
/// is what a stricter test here would turn a typo into.
pub fn reported(line: &str) -> bool {
    let trimmed = line.trim();
    [PREFIX, DECODED].iter().any(|word| {
        match trimmed.strip_prefix(*word) {
            // The bare word is a report too, and a malformed one. Reading it as prose instead
            // would turn a smoke that printed the word and nothing else into a silence.
            Some(rest) => rest.is_empty() || rest.starts_with(char::is_whitespace),
            None => false,
        }
    })
}

/// What one report says, or a sentence saying what the line is missing.
pub fn report(line: &str) -> Result<Report, String> {
    let mut words = line.split_whitespace();
    match words.next() {
        Some(word) if word == PREFIX => exercised(words),
        Some(word) if word == DECODED => decoded(words),
        _ => Err(format!("it opens with neither `{PREFIX}` nor `{DECODED}`")),
    }
}

/// `exercised <operationId> <outcome>`, and nothing else.
fn exercised<'a>(mut words: impl Iterator<Item = &'a str>) -> Result<Report, String> {
    let operation = words
        .next()
        .ok_or_else(|| "it names no operation".to_owned())?;
    let outcome = words
        .next()
        .ok_or_else(|| format!("it says neither `{ACCEPTED}` nor `{REFUSED}<problemId>`"))?;
    if words.next().is_some() {
        return Err(format!(
            "it carries more than `{PREFIX} <operationId> <outcome>`"
        ));
    }

    let outcome = if outcome == ACCEPTED {
        Outcome::Accepted
    } else if let Some(problem) = outcome.strip_prefix(REFUSED) {
        if problem.is_empty() {
            return Err(format!("`{REFUSED}` names no problem"));
        }
        Outcome::Refused(problem.to_owned())
    } else {
        return Err(format!(
            "`{outcome}` is neither `{ACCEPTED}` nor `{REFUSED}<problemId>`"
        ));
    };

    Ok(Report::Exercised(operation.to_owned(), outcome))
}

/// `decoded <ModelName>`, and nothing else.
fn decoded<'a>(mut words: impl Iterator<Item = &'a str>) -> Result<Report, String> {
    let model = words.next().ok_or_else(|| "it names no model".to_owned())?;
    if words.next().is_some() {
        return Err(format!("it carries more than `{DECODED} <ModelName>`"));
    }
    Ok(Report::Decoded(model.to_owned()))
}

/// Every operation a client carrying that tag is generated from.
///
/// Read through the generator rather than out of the document by hand, and narrowed by the tag the
/// generator's own registry says the target selects, so that the set a language is held to and the
/// set it was generated from cannot come apart. The tag is a parameter rather than a constant here
/// because the registry already answers it per target: the eleven SDKs select the public surface
/// and the MCP server selects its own, and a set written down here would make that a special case.
pub fn declared(snapshot: &Path, tag: &str) -> Result<BTreeSet<String>, Error> {
    let read =
        Snapshot::from_path(snapshot, tag, &Limits::DEFAULT).map_err(|cause| Error::Document {
            path: snapshot.display().to_string(),
            detail: format!("{cause}"),
        })?;

    let mut names = BTreeSet::new();
    for operation in read.operations() {
        match &operation.operation_id {
            Some(id) => {
                names.insert(id.clone());
            }
            // No id is no name to report, and an operation nothing can name is one no language
            // could be held to. The generator cannot emit it either, so this is the document
            // being wrong rather than a case to pass over.
            None => {
                return Err(Error::Document {
                    path: snapshot.display().to_string(),
                    detail: format!(
                        "`{}` carries `{tag}` and declares no operationId, so nothing can name it",
                        operation.location()
                    ),
                });
            }
        }
    }

    if names.is_empty() {
        return Err(Error::Document {
            path: snapshot.display().to_string(),
            detail: format!("it declares no operation carrying `{tag}`"),
        });
    }

    Ok(names)
}

/// What the generator emits for a target, and which of those an operation really answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Models {
    /// Every named type the generator emits: one per component schema, one per object written
    /// where it is used, one per closed list of strings.
    pub emitted: BTreeSet<String>,
    /// The ones an operation answers, reached from a success body through fields the document says
    /// are always there.
    ///
    /// Narrower than [`Models::emitted`], and every step of the narrowing is the document's rather
    /// than this crate's. A type only a request body carries is never decoded by anybody — what
    /// would catch a wrong field name there is the operation that sends it being refused, which the
    /// other bijection already holds. A type reachable only through a field the document marks
    /// optional is decoded only by an instance that populates it, so holding every instance to it
    /// would be holding the run to a configuration rather than to a client. Both are reported at
    /// the end of a run rather than written down anywhere.
    pub answered: BTreeSet<String>,
}

/// Every model type a client carrying that tag is generated from, and which of them are decodable.
///
/// `decoding` is what stops this holding a client to types it was never given. The tag says which
/// operations a target covers and says nothing about whether that target writes their types, so
/// deriving the set from the tag alone demands twenty-nine models of a client that emits none and
/// hands its answers on untouched. For the eleven SDKs the two readings coincide, which is exactly
/// why the mistake stays invisible until a target arrives where they do not.
pub fn models(snapshot: &Path, tag: &str, decoding: Decoding) -> Result<Models, Error> {
    // A target that writes no types decodes none, and there is nothing to derive: an empty pair
    // here is what the rest of this module then holds it to, which is nothing. Returned before the
    // document is read rather than after, so that the emptiness is a stated property of the target
    // and not something the snapshot happened to produce.
    if decoding == Decoding::PassThrough {
        return Ok(Models {
            emitted: BTreeSet::new(),
            answered: BTreeSet::new(),
        });
    }

    let refuse = |detail: String| Error::Document {
        path: snapshot.display().to_string(),
        detail,
    };

    let read = Snapshot::from_path(snapshot, tag, &Limits::DEFAULT)
        .map_err(|cause| refuse(format!("{cause}")))?;
    let model = ApiModel::from_snapshot(&read, &Limits::DEFAULT)
        .map_err(|cause| refuse(format!("{cause}")))?;
    let enumerations = model
        .enumerations(&Limits::DEFAULT)
        .map_err(|cause| refuse(format!("{cause}")))?;

    let mut emitted: BTreeSet<String> = model.schemas.keys().cloned().collect();
    emitted.extend(enumerations.into_keys());

    let mut answered = BTreeSet::new();
    for entity in model.entities.entities() {
        for method in &entity.methods {
            if let Some((_, Some(shape))) = method.success.as_ref() {
                walk(shape, &model.schemas, &mut answered, 0)?;
            }
        }
    }

    if emitted.is_empty() {
        return Err(refuse(format!("it declares no model under `{tag}`")));
    }

    Ok(Models { emitted, answered })
}

/// Gathers every named type reachable from one shape, through fields that are always there.
///
/// Arrays and maps are walked through: what an operation answers a list of is what a caller
/// decodes, one element at a time. Optional fields are not, for the reason [`Models::answered`]
/// gives.
fn walk(
    shape: &Shape,
    schemas: &std::collections::BTreeMap<String, ObjectShape>,
    found: &mut BTreeSet<String>,
    depth: usize,
) -> Result<(), Error> {
    // The same ceiling the generator reads a shape under, so a document this walk gives up on is
    // one the generator would have given up on first.
    if depth > Limits::DEFAULT.max_shape_depth {
        return Ok(());
    }

    match shape {
        Shape::Scalar(_) | Shape::Json => Ok(()),
        Shape::Array(inner) | Shape::Map(inner) => walk(inner, schemas, found, depth + 1),
        Shape::Enum { name, .. } => {
            found.insert(name.clone());
            Ok(())
        }
        Shape::Object(object) => {
            // Already seen means already walked: a type reached twice is one type, and following
            // it again is what turns a document that refers to itself into a walk that never ends.
            if !found.insert(object.name.clone()) {
                return Ok(());
            }
            fields(object, schemas, found, depth)
        }
        Shape::Named(name) => {
            if !found.insert(name.clone()) {
                return Ok(());
            }
            match schemas.get(name) {
                Some(object) => fields(object, schemas, found, depth),
                // A name the registry does not carry is a reference the generator resolved
                // somewhere this walk cannot follow; naming it is still right, since a client
                // decodes it.
                None => Ok(()),
            }
        }
    }
}

/// Walks the fields of one object that the document says are always there.
fn fields(
    object: &ObjectShape,
    schemas: &std::collections::BTreeMap<String, ObjectShape>,
    found: &mut BTreeSet<String>,
    depth: usize,
) -> Result<(), Error> {
    for field in &object.fields {
        if field.required {
            walk(&field.shape, schemas, found, depth + 1)?;
        }
    }
    Ok(())
}

/// What one language proved, once its reports have been held to what the document declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    pub operations: usize,
    pub models: usize,
}

/// Holds one language's reports to what the document declares, on both counts.
///
/// `drives` is what the smoke's own manifest says about itself, and it is the only thing that tells
/// a language still to be ported from one that has stopped reporting. A smoke that says it does not
/// drive the surface and then reports is refused, and so is one that says it does and reports
/// nothing — so the two halves cannot drift apart, and a language cannot be un-ported quietly.
pub fn held(
    target: &str,
    drives: bool,
    declared: &BTreeSet<String>,
    models: &Models,
    reported: &[String],
) -> Result<Held, Error> {
    if reported.len() > MAX_REPORTS {
        return Err(Error::TooManyReports {
            target: target.to_owned(),
            maximum: MAX_REPORTS,
        });
    }

    if !drives {
        if reported.is_empty() {
            return Ok(Held {
                operations: 0,
                models: 0,
            });
        }
        return Err(Error::SurfaceUnannounced {
            target: target.to_owned(),
            reports: reported.len(),
        });
    }

    if reported.is_empty() {
        return Err(Error::SurfaceSilent {
            target: target.to_owned(),
        });
    }

    let mut outcomes: BTreeMap<String, Outcome> = BTreeMap::new();
    let mut decoded: BTreeSet<String> = BTreeSet::new();
    for line in reported {
        let read = report(line).map_err(|detail| Error::Unreportable {
            target: target.to_owned(),
            line: line.clone(),
            detail,
        })?;

        match read {
            Report::Decoded(model) => {
                decoded.insert(model);
            }
            Report::Exercised(operation, outcome) => {
                // The same operation twice with the same answer is a flow that read a list before
                // and after it changed something, which is ordinary. Twice with two answers is one
                // call site reporting what another call site did.
                if let Some(first) = outcomes.get(&operation)
                    && *first != outcome
                {
                    return Err(Error::SurfaceAmbiguous {
                        target: target.to_owned(),
                        operation,
                        first: format!("{first}"),
                        second: format!("{outcome}"),
                    });
                }
                outcomes.insert(operation, outcome);
            }
        }
    }

    let throttled: Vec<String> = outcomes
        .iter()
        .filter(|(_, outcome)| **outcome == Outcome::Refused(THROTTLED.to_owned()))
        .map(|(operation, _)| operation.clone())
        .collect();
    if !throttled.is_empty() {
        return Err(Error::SurfaceThrottled {
            target: target.to_owned(),
            throttled,
        });
    }

    let unknown: Vec<String> = outcomes
        .keys()
        .filter(|operation| !declared.contains(*operation))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        return Err(Error::SurfaceUnknown {
            target: target.to_owned(),
            unknown,
        });
    }

    // Held against everything the generator emits rather than against what an operation answers:
    // a model this run cannot reach is still a model, and a smoke on an instance that populates one
    // of the optional ones is right to say so. What this catches is a name that is not a model at
    // all, which is what a typo looks like from here.
    let unnamed: Vec<String> = decoded
        .iter()
        .filter(|model| !models.emitted.contains(*model))
        .cloned()
        .collect();
    if !unnamed.is_empty() {
        return Err(Error::ModelsUnknown {
            target: target.to_owned(),
            unknown: unnamed,
        });
    }

    let missing: Vec<String> = declared
        .iter()
        .filter(|operation| !outcomes.contains_key(*operation))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(Error::SurfaceMissing {
            target: target.to_owned(),
            missing,
            declared: declared.len(),
        });
    }

    let undecoded: Vec<String> = models
        .answered
        .iter()
        .filter(|model| !decoded.contains(*model))
        .cloned()
        .collect();
    if !undecoded.is_empty() {
        return Err(Error::ModelsMissing {
            target: target.to_owned(),
            missing: undecoded,
            answered: models.answered.len(),
        });
    }

    Ok(Held {
        operations: outcomes.len(),
        models: decoded.len(),
    })
}

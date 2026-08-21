//! The error contract every generated client answers a failure with, discovered from the document.
//!
//! Nothing here names the schema it is looking for. The API is free to rename it, and a client
//! generated after the rename still reads failures whole, because the schema is found by what the
//! operations point at rather than by a name written down in this crate: the one schema every
//! failing response of every selected operation carries. The list of problems it can report is
//! found the same way — the single closed string enum that schema declares.
//!
//! Both discoveries refuse ambiguity. A document where operations disagree, or where the schema
//! declares two closed enums, is one where a generated client would pick the wrong one silently.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::{Error, preview};
use crate::model::shape::{ObjectShape, Shape, referenced_name};
use crate::snapshot::{Operation, ResponseBody, ResponseStatus};

/// Lowest status an operation answers a failure with.
const FAILURE_STATUS: u16 = 400;

/// How the candidates of an ambiguous discovery are joined into one message.
const CANDIDATE_SEPARATOR: &str = ", ";

/// The problems the API reports, as the error schema lists them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemCatalogue {
    values: Vec<String>,
}

impl ProblemCatalogue {
    /// The problems, ordered and deduplicated. Never empty: a catalogue with nothing in it is not
    /// a catalogue, and the discovery refuses one.
    pub fn values(&self) -> &[String] {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// What a generated client reads out of a failing response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorModel {
    /// Name the document declares the error schema under.
    pub schema: String,
    pub shape: ObjectShape,
    /// Field of the schema that tells one problem from another.
    pub discriminant: String,
    pub catalogue: ProblemCatalogue,
    /// Statuses the selected operations answer failures with, ordered.
    pub statuses: Vec<u16>,
}

impl ErrorModel {
    /// Finds the error contract the selected operations share.
    pub(crate) fn discover(
        operations: &[Operation],
        schemas: &BTreeMap<String, ObjectShape>,
    ) -> Result<Self, Error> {
        let mut candidates: BTreeSet<String> = BTreeSet::new();
        let mut statuses: BTreeSet<u16> = BTreeSet::new();

        for operation in operations {
            for response in &operation.responses {
                let ResponseStatus::Code(status) = response.status else {
                    continue;
                };
                if status < FAILURE_STATUS {
                    continue;
                }

                // A failure carrying no body, or one in a media type no target reads, says nothing
                // about the contract: it is the named JSON bodies that describe it.
                statuses.insert(status);

                let Some(ResponseBody::Json(schema)) = response.body.as_ref() else {
                    continue;
                };

                let name = referenced_name(schema).ok_or_else(|| Error::UnnamedErrorSchema {
                    operation: preview(&subject(operation)),
                })?;
                candidates.insert(name);
            }
        }

        let candidates: Vec<String> = candidates.into_iter().collect();
        let [schema] = candidates.as_slice() else {
            if candidates.is_empty() {
                return Err(Error::UndiscoverableErrorSchema {
                    threshold: FAILURE_STATUS,
                });
            }
            return Err(Error::DisagreeingErrorSchemas {
                candidates: preview(&candidates.join(CANDIDATE_SEPARATOR)),
            });
        };
        let schema = schema.clone();

        let shape = schemas
            .get(&schema)
            .ok_or_else(|| Error::UnresolvableReference {
                reference: preview(&schema),
            })?
            .clone();
        let (discriminant, catalogue) = catalogue_of(&shape)?;

        Ok(Self {
            schema,
            shape,
            discriminant,
            catalogue,
            statuses: statuses.into_iter().collect(),
        })
    }
}

/// The single member of the error schema listing the problems it can report.
fn catalogue_of(shape: &ObjectShape) -> Result<(String, ProblemCatalogue), Error> {
    let mut closed = shape.fields.iter().filter_map(|field| match &field.shape {
        Shape::Enum { values, .. } => Some((field.name.clone(), values.clone())),
        _ => None,
    });

    let Some((discriminant, values)) = closed.next() else {
        return Err(Error::ErrorSchemaWithoutCatalogue {
            schema: preview(&shape.name),
        });
    };

    let others: Vec<String> = closed.map(|(name, _)| name).collect();
    if !others.is_empty() {
        let mut members = vec![discriminant];
        members.extend(others);
        return Err(Error::AmbiguousErrorCatalogue {
            schema: preview(&shape.name),
            members: preview(&members.join(CANDIDATE_SEPARATOR)),
        });
    }

    let values: BTreeSet<String> = values.into_iter().collect();

    Ok((
        discriminant,
        ProblemCatalogue {
            values: values.into_iter().collect(),
        },
    ))
}

/// How an operation is named in a discovery message.
fn subject(operation: &Operation) -> String {
    operation
        .operation_id
        .clone()
        .unwrap_or_else(|| operation.location())
}

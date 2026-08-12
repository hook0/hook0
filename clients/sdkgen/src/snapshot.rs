use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use openapiv3::{
    Components, OpenAPI, Operation as SpecOperation, Parameter as SpecParameter, PathItem,
    ReferenceOr,
};

use crate::error::{Error, preview};
use crate::limits::Limits;

/// Tag an operation carries to be part of the surface SDKs expose.
pub const PUBLIC_TAG: &str = "public";

/// Where a parameter reference is looked up.
const PARAMETER_REFERENCE_PREFIX: &str = "#/components/parameters/";

/// An HTTP method an operation answers on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HttpMethod {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Put => "PUT",
            Self::Post => "POST",
            Self::Delete => "DELETE",
            Self::Options => "OPTIONS",
            Self::Head => "HEAD",
            Self::Patch => "PATCH",
            Self::Trace => "TRACE",
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Where a parameter travels in the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

/// A single parameter of an operation, path-level ones included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub location: ParameterLocation,
    pub required: bool,
}

/// One operation of the snapshot, kept as the spec declares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    /// Absent when the spec declares no `operationId`, which no entity can be derived from.
    pub operation_id: Option<String>,
    pub method: HttpMethod,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub parameters: Vec<Parameter>,
}

impl Operation {
    /// How the operation is named in error messages, when it has no usable id.
    pub fn location(&self) -> String {
        format!("{} {}", self.method, self.path)
    }
}

/// The operations an OpenAPI snapshot exposes, once its ceilings have been honoured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    title: String,
    version: String,
    operations: Vec<Operation>,
    public_tag_applied: bool,
    filtered_out: usize,
}

impl Snapshot {
    /// Reads a snapshot from disk, refusing files above the byte ceiling before reading them.
    pub fn from_path(path: &Path, limits: &Limits) -> Result<Self, Error> {
        let metadata = fs::metadata(path).map_err(|err| Error::SnapshotUnreadable {
            path: path.display().to_string(),
            reason: err.to_string(),
        })?;

        if metadata.len() > limits.max_snapshot_bytes as u64 {
            return Err(Error::SnapshotTooLarge {
                size: metadata.len(),
                limit: limits.max_snapshot_bytes,
            });
        }

        let bytes = fs::read(path).map_err(|err| Error::SnapshotUnreadable {
            path: path.display().to_string(),
            reason: err.to_string(),
        })?;

        Self::from_bytes(&bytes, limits)
    }

    /// Parses a snapshot held in memory.
    pub fn from_bytes(bytes: &[u8], limits: &Limits) -> Result<Self, Error> {
        if bytes.len() > limits.max_snapshot_bytes {
            return Err(Error::SnapshotTooLarge {
                size: bytes.len() as u64,
                limit: limits.max_snapshot_bytes,
            });
        }

        let spec: OpenAPI = serde_json::from_slice(bytes)
            .map_err(|err| Error::MalformedSnapshot(preview(&err.to_string())))?;

        Self::from_spec(&spec, limits)
    }

    fn from_spec(spec: &OpenAPI, limits: &Limits) -> Result<Self, Error> {
        let declared = count_operations(spec)?;
        if declared > limits.max_operations {
            return Err(Error::TooManyOperations {
                count: declared,
                limit: limits.max_operations,
            });
        }

        let mut operations = Vec::with_capacity(declared);
        let mut seen_ids: HashMap<String, String> = HashMap::with_capacity(declared);

        for (path, item) in spec.paths.iter() {
            let item = path_item(item)?;
            let shared = read_parameters(&item.parameters, spec.components.as_ref(), limits)?;

            for (method, spec_operation) in operations_of(item) {
                let operation = read_operation(
                    method,
                    path,
                    spec_operation,
                    &shared,
                    spec.components.as_ref(),
                    limits,
                )?;

                if let Some(operation_id) = operation.operation_id.as_ref()
                    && let Some(first) = seen_ids.insert(operation_id.clone(), operation.location())
                {
                    return Err(Error::DuplicateOperationId {
                        operation_id: preview(operation_id),
                        first,
                        second: operation.location(),
                    });
                }

                operations.push(operation);
            }
        }

        let public_tag_applied = operations
            .iter()
            .any(|operation| operation.tags.iter().any(|tag| tag == PUBLIC_TAG));

        let declared_count = operations.len();
        if public_tag_applied {
            operations.retain(|operation| operation.tags.iter().any(|tag| tag == PUBLIC_TAG));
        }

        Ok(Self {
            title: spec.info.title.clone(),
            version: spec.info.version.clone(),
            filtered_out: declared_count - operations.len(),
            operations,
            public_tag_applied,
        })
    }

    /// Title the spec gives itself.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Version the spec gives itself.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Operations SDKs are built from, in document order.
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    /// Whether the spec carries the public tag, which narrows the snapshot down to what it marks.
    pub fn public_tag_applied(&self) -> bool {
        self.public_tag_applied
    }

    /// How many operations the public tag left out.
    pub fn filtered_out(&self) -> usize {
        self.filtered_out
    }
}

/// Pairs each operation of a path item with the method it answers on.
fn operations_of(item: &PathItem) -> impl Iterator<Item = (HttpMethod, &SpecOperation)> {
    [
        (HttpMethod::Get, &item.get),
        (HttpMethod::Put, &item.put),
        (HttpMethod::Post, &item.post),
        (HttpMethod::Delete, &item.delete),
        (HttpMethod::Options, &item.options),
        (HttpMethod::Head, &item.head),
        (HttpMethod::Patch, &item.patch),
        (HttpMethod::Trace, &item.trace),
    ]
    .into_iter()
    .filter_map(|(method, operation)| operation.as_ref().map(|operation| (method, operation)))
}

fn count_operations(spec: &OpenAPI) -> Result<usize, Error> {
    let mut count = 0;
    for (_, item) in spec.paths.iter() {
        count += operations_of(path_item(item)?).count();
    }
    Ok(count)
}

/// OpenAPI 3.0 has nowhere to resolve a referenced path item from, so one is rejected outright.
fn path_item(entry: &ReferenceOr<PathItem>) -> Result<&PathItem, Error> {
    match entry {
        ReferenceOr::Item(item) => Ok(item),
        ReferenceOr::Reference { reference } => Err(Error::UnresolvableReference {
            reference: preview(reference),
        }),
    }
}

fn read_operation(
    method: HttpMethod,
    path: &str,
    operation: &SpecOperation,
    shared: &[Parameter],
    components: Option<&Components>,
    limits: &Limits,
) -> Result<Operation, Error> {
    let operation_id = match operation.operation_id.as_ref() {
        Some(operation_id) => Some(bounded_identifier(operation_id, limits)?),
        None => None,
    };

    let own = read_parameters(&operation.parameters, components, limits)?;
    let parameters = merge_parameters(shared, own);

    if parameters.len() > limits.max_parameters_per_operation {
        return Err(Error::TooManyParameters {
            method: method.to_string(),
            path: preview(path),
            count: parameters.len(),
            limit: limits.max_parameters_per_operation,
        });
    }

    Ok(Operation {
        operation_id,
        method,
        path: path.to_owned(),
        summary: operation.summary.clone(),
        description: operation.description.clone(),
        tags: operation.tags.clone(),
        parameters,
    })
}

/// Operation-level parameters win over path-level ones sharing their name and location.
fn merge_parameters(shared: &[Parameter], own: Vec<Parameter>) -> Vec<Parameter> {
    let mut merged: Vec<Parameter> = shared
        .iter()
        .filter(|candidate| {
            !own.iter().any(|parameter| {
                parameter.name == candidate.name && parameter.location == candidate.location
            })
        })
        .cloned()
        .collect();

    merged.extend(own);
    merged
}

fn read_parameters(
    entries: &[ReferenceOr<SpecParameter>],
    components: Option<&Components>,
    limits: &Limits,
) -> Result<Vec<Parameter>, Error> {
    entries
        .iter()
        .map(|entry| read_parameter(entry, components, limits))
        .collect()
}

fn read_parameter(
    entry: &ReferenceOr<SpecParameter>,
    components: Option<&Components>,
    limits: &Limits,
) -> Result<Parameter, Error> {
    let parameter = resolve_parameter(entry, components, limits)?;
    let data = parameter.parameter_data_ref();

    let location = match parameter {
        SpecParameter::Path { .. } => ParameterLocation::Path,
        SpecParameter::Query { .. } => ParameterLocation::Query,
        SpecParameter::Header { .. } => ParameterLocation::Header,
        SpecParameter::Cookie { .. } => ParameterLocation::Cookie,
    };

    Ok(Parameter {
        name: bounded_identifier(&data.name, limits)?,
        location,
        required: data.required,
    })
}

/// Follows `$ref` hops until a parameter is reached, which is also what stops a reference cycle.
fn resolve_parameter<'a>(
    entry: &'a ReferenceOr<SpecParameter>,
    components: Option<&'a Components>,
    limits: &Limits,
) -> Result<&'a SpecParameter, Error> {
    let mut current = entry;
    let mut hops = 0;

    loop {
        match current {
            ReferenceOr::Item(parameter) => return Ok(parameter),
            ReferenceOr::Reference { reference } => {
                if hops >= limits.max_reference_depth {
                    return Err(Error::ReferenceTooDeep {
                        reference: preview(reference),
                        limit: limits.max_reference_depth,
                    });
                }
                hops += 1;

                let name = reference
                    .strip_prefix(PARAMETER_REFERENCE_PREFIX)
                    .map(decode_pointer_segment)
                    .ok_or_else(|| Error::UnresolvableReference {
                        reference: preview(reference),
                    })?;

                current = components
                    .and_then(|components| components.parameters.get(&name))
                    .ok_or_else(|| Error::UnresolvableReference {
                        reference: preview(reference),
                    })?;
            }
        }
    }
}

/// Turns a JSON pointer segment back into the key it names.
fn decode_pointer_segment(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

fn bounded_identifier(identifier: &str, limits: &Limits) -> Result<String, Error> {
    if identifier.len() > limits.max_identifier_bytes {
        return Err(Error::IdentifierTooLong {
            identifier: preview(identifier),
            size: identifier.len(),
            limit: limits.max_identifier_bytes,
        });
    }

    Ok(identifier.to_owned())
}

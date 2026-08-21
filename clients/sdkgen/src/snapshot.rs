use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs;
use std::path::Path;

use openapiv3::{
    Components, OpenAPI, Operation as SpecOperation, Parameter as SpecParameter,
    ParameterData as SpecParameterData, ParameterSchemaOrContent, PathItem, ReferenceOr,
    RequestBody as SpecRequestBody, Response as SpecResponse, Schema, StatusCode,
};
use serde_json::Value;

use crate::error::{Error, preview};
use crate::limits::Limits;
// The vocabulary a schema may be written in is one question, asked of a body, of a component and
// of a parameter alike, so it is answered in one place rather than three.
use crate::model::shape::{IGNORED_KEYWORDS, MODELLED_KEYWORDS};

/// Tag an operation carries to be part of the surface SDKs expose.
pub const SDK_TAG: &str = "sdk";

/// Where a parameter reference is looked up.
const PARAMETER_REFERENCE_PREFIX: &str = "#/components/parameters/";

/// Where a request body reference is looked up.
const REQUEST_BODY_REFERENCE_PREFIX: &str = "#/components/requestBodies/";

/// Where a schema reference is looked up.
pub(crate) const SCHEMA_REFERENCE_PREFIX: &str = "#/components/schemas/";

/// Where a response reference is looked up.
const RESPONSE_REFERENCE_PREFIX: &str = "#/components/responses/";

/// Where a security scheme reference is looked up.
const SECURITY_SCHEME_REFERENCE_PREFIX: &str = "#/components/securitySchemes/";

/// Media type a request body is read from.
const JSON_MEDIA_TYPE: &str = "application/json";

/// JSON type a parameter falls back to when the document names none.
const DEFAULT_PARAMETER_TYPE: &str = "string";

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
    pub description: Option<String>,
    /// JSON type the document gives the parameter, [`DEFAULT_PARAMETER_TYPE`] when it names none.
    pub schema_type: String,
}

/// What an operation reads in its request body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestBody {
    /// A JSON body, its schema resolved down to what the document declares.
    Json(Value),
    /// A body in a media type none of the targets describes.
    Other,
}

/// What an operation writes back under one status, kept as the spec declares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseBody {
    /// A JSON body, its schema left exactly as the document writes it so a `$ref` still names the
    /// component it points at.
    Json(Value),
    /// A body in a media type none of the targets describes.
    Other,
}

/// The status an operation answers a response under, as the document writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResponseStatus {
    /// A single status code, as `"200"` writes it.
    Code(u16),
    /// A class of status codes, as `"2XX"` writes it, carrying the leading digit alone.
    Range(u16),
    /// The `default` entry, which stands for whatever status the document does not spell out.
    Default,
}

/// One response of an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: ResponseStatus,
    /// Absent when the response carries no content at all.
    pub body: Option<ResponseBody>,
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
    /// Absent when the operation reads no body.
    pub request_body: Option<RequestBody>,
    /// Component the JSON request body was reached through, absent when the body is written where
    /// the operation declares it. [`RequestBody::Json`] carries the schema with its references
    /// already followed, which is what a tool caller needs and what loses that name.
    pub request_body_schema: Option<String>,
    /// Responses the operation declares, ordered by status.
    pub responses: Vec<Response>,
    /// Security schemes the operation requires, named as `components.securitySchemes` declares
    /// them. Alternatives are flattened: a target that carries every named scheme satisfies the
    /// operation whichever way the document groups them.
    pub security: Vec<String>,
}

impl Operation {
    /// How the operation is named in error messages, when it has no usable id.
    pub fn location(&self) -> String {
        format!("{} {}", self.method, self.path)
    }

    /// Whether the operation carries that tag.
    fn carries(&self, tag: &str) -> bool {
        self.tags.iter().any(|carried| carried == tag)
    }
}

/// The operations an OpenAPI snapshot exposes, once its ceilings have been honoured.
///
/// The snapshot stays shaped like the document it came from: schemas and security schemes are the
/// JSON the spec writes, references included. Turning them into the type language targets are
/// generated from is what [`crate::ApiModel`] is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    title: String,
    version: String,
    servers: Vec<String>,
    operations: Vec<Operation>,
    schemas: BTreeMap<String, Value>,
    security_schemes: BTreeMap<String, Value>,
    tag_applied: bool,
    filtered_out: usize,
}

impl Snapshot {
    /// Reads a snapshot from disk, refusing files above the byte ceiling before reading them.
    pub fn from_path(path: &Path, tag: &str, limits: &Limits) -> Result<Self, Error> {
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

        Self::from_bytes(&bytes, tag, limits)
    }

    /// Parses a snapshot held in memory, narrowed to the operations a target selects.
    pub fn from_bytes(bytes: &[u8], tag: &str, limits: &Limits) -> Result<Self, Error> {
        if bytes.len() > limits.max_snapshot_bytes {
            return Err(Error::SnapshotTooLarge {
                size: bytes.len() as u64,
                limit: limits.max_snapshot_bytes,
            });
        }

        let spec: OpenAPI = serde_json::from_slice(bytes)
            .map_err(|err| Error::MalformedSnapshot(preview(&err.to_string())))?;

        Self::from_spec(&spec, tag, limits)
    }

    fn from_spec(spec: &OpenAPI, tag: &str, limits: &Limits) -> Result<Self, Error> {
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

        let tag_applied = operations.iter().any(|operation| operation.carries(tag));

        let declared_count = operations.len();
        if tag_applied {
            operations.retain(|operation| operation.carries(tag));
        }

        Ok(Self {
            title: spec.info.title.clone(),
            version: spec.info.version.clone(),
            servers: spec
                .servers
                .iter()
                .map(|server| server.url.clone())
                .collect(),
            schemas: read_component_schemas(spec.components.as_ref(), limits)?,
            security_schemes: read_security_schemes(spec.components.as_ref(), limits)?,
            filtered_out: declared_count - operations.len(),
            operations,
            tag_applied,
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

    /// Base URLs the spec serves its operations from.
    pub fn servers(&self) -> &[String] {
        &self.servers
    }

    /// Schemas `components.schemas` declares, as the document writes them.
    pub fn schemas(&self) -> &BTreeMap<String, Value> {
        &self.schemas
    }

    /// Security schemes `components.securitySchemes` declares, as the document writes them.
    pub fn security_schemes(&self) -> &BTreeMap<String, Value> {
        &self.security_schemes
    }

    /// Operations the target is built from, in document order.
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    /// Whether the spec carries that tag, which narrows the snapshot down to what it marks.
    pub fn tag_applied(&self) -> bool {
        self.tag_applied
    }

    /// How many operations the tag left out.
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

    let subject = operation_id
        .clone()
        .unwrap_or_else(|| format!("{method} {path}"));

    let (request_body, request_body_schema) =
        read_request_body(operation, &subject, components, limits)?;

    Ok(Operation {
        operation_id,
        method,
        path: path.to_owned(),
        summary: operation.summary.clone(),
        description: operation.description.clone(),
        tags: operation.tags.clone(),
        parameters,
        request_body,
        request_body_schema,
        responses: read_responses(operation, &subject, components, limits)?,
        security: read_security(operation, limits)?,
    })
}

/// The body an operation reads, with the reference reaching its JSON schema resolved, alongside
/// the name of the component that reference went through.
fn read_request_body(
    operation: &SpecOperation,
    subject: &str,
    components: Option<&Components>,
    limits: &Limits,
) -> Result<(Option<RequestBody>, Option<String>), Error> {
    let Some(body) = operation.request_body.as_ref() else {
        return Ok((None, None));
    };

    let body: &SpecRequestBody = resolve(
        body,
        REQUEST_BODY_REFERENCE_PREFIX,
        |name| components.and_then(|components| components.request_bodies.get(name)),
        limits,
    )?;

    let Some(schema) = body
        .content
        .get(JSON_MEDIA_TYPE)
        .and_then(|content| content.schema.as_ref())
    else {
        return Ok((Some(RequestBody::Other), None));
    };

    let (schema, schema_name): (&Schema, Option<String>) = resolve_named(
        schema,
        SCHEMA_REFERENCE_PREFIX,
        |name| components.and_then(|components| components.schemas.get(name)),
        limits,
    )?;

    let schema = serde_json::to_value(schema).map_err(|err| Error::UnserializableSchema {
        subject: preview(subject),
        reason: err.to_string(),
    })?;

    Ok((Some(RequestBody::Json(schema)), schema_name))
}

/// The responses an operation declares, with the reference reaching each of them resolved.
///
/// The schema a response carries is kept exactly as the document writes it: a target that has to
/// declare a type for an error body needs the name of the component it points at, and following
/// the reference here is what would drop it.
fn read_responses(
    operation: &SpecOperation,
    subject: &str,
    components: Option<&Components>,
    limits: &Limits,
) -> Result<Vec<Response>, Error> {
    let declared = operation
        .responses
        .default
        .as_ref()
        .map(|response| (ResponseStatus::Default, response))
        .into_iter()
        .chain(
            operation
                .responses
                .responses
                .iter()
                .map(|(status, response)| (response_status(status), response)),
        );

    let mut responses = Vec::new();
    for (status, response) in declared {
        let response: &SpecResponse = resolve(
            response,
            RESPONSE_REFERENCE_PREFIX,
            |name| components.and_then(|components| components.responses.get(name)),
            limits,
        )?;

        responses.push(Response {
            status,
            body: read_response_body(response, subject)?,
        });
    }

    responses.sort_by_key(|response| response.status);
    Ok(responses)
}

fn read_response_body(
    response: &SpecResponse,
    subject: &str,
) -> Result<Option<ResponseBody>, Error> {
    if response.content.is_empty() {
        return Ok(None);
    }

    let Some(schema) = response
        .content
        .get(JSON_MEDIA_TYPE)
        .and_then(|content| content.schema.as_ref())
    else {
        return Ok(Some(ResponseBody::Other));
    };

    serde_json::to_value(schema)
        .map(ResponseBody::Json)
        .map(Some)
        .map_err(|err| Error::UnserializableSchema {
            subject: preview(subject),
            reason: err.to_string(),
        })
}

fn response_status(status: &StatusCode) -> ResponseStatus {
    match status {
        StatusCode::Code(code) => ResponseStatus::Code(*code),
        StatusCode::Range(range) => ResponseStatus::Range(*range),
    }
}

/// The security schemes an operation requires, in the order the document names them.
///
/// A document groups requirements as alternatives, each one a set of schemes to satisfy together.
/// Generated clients carry credentials rather than pick between them, so the names are flattened
/// into the set an operation may be called with.
fn read_security(operation: &SpecOperation, limits: &Limits) -> Result<Vec<String>, Error> {
    let Some(requirements) = operation.security.as_ref() else {
        return Ok(Vec::new());
    };

    let mut names = Vec::new();
    for requirement in requirements {
        for name in requirement.keys() {
            let name = bounded_identifier(name, limits)?;
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }

    Ok(names)
}

fn read_component_schemas(
    components: Option<&Components>,
    limits: &Limits,
) -> Result<BTreeMap<String, Value>, Error> {
    let Some(components) = components else {
        return Ok(BTreeMap::new());
    };

    let mut schemas = BTreeMap::new();
    for (name, schema) in &components.schemas {
        let schema: &Schema = resolve(
            schema,
            SCHEMA_REFERENCE_PREFIX,
            |name| components.schemas.get(name),
            limits,
        )?;
        let schema = serde_json::to_value(schema).map_err(|err| Error::UnserializableSchema {
            subject: preview(name),
            reason: err.to_string(),
        })?;
        schemas.insert(bounded_identifier(name, limits)?, schema);
    }

    Ok(schemas)
}

fn read_security_schemes(
    components: Option<&Components>,
    limits: &Limits,
) -> Result<BTreeMap<String, Value>, Error> {
    let Some(components) = components else {
        return Ok(BTreeMap::new());
    };

    let mut schemes = BTreeMap::new();
    for (name, scheme) in &components.security_schemes {
        let scheme = resolve(
            scheme,
            SECURITY_SCHEME_REFERENCE_PREFIX,
            |name| components.security_schemes.get(name),
            limits,
        )?;
        let scheme = serde_json::to_value(scheme).map_err(|err| Error::UnserializableSchema {
            subject: preview(name),
            reason: err.to_string(),
        })?;
        schemes.insert(bounded_identifier(name, limits)?, scheme);
    }

    Ok(schemes)
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
    let parameter: &SpecParameter = resolve(
        entry,
        PARAMETER_REFERENCE_PREFIX,
        |name| components.and_then(|components| components.parameters.get(name)),
        limits,
    )?;
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
        description: data.description.clone(),
        schema_type: parameter_type(data)?,
    })
}

/// The JSON type a parameter declares, which a caller has to fill in.
///
/// A parameter that names no type, or reaches its schema through a reference, falls back to
/// [`DEFAULT_PARAMETER_TYPE`] rather than being dropped.
///
/// What the schema says beyond its type is not modelled here, and saying nothing about it is not
/// the same as accepting it. A body and a component are held to the vocabulary the reader
/// understands, and a parameter was not. `{"type": "string", "nullable": true}` was read as a
/// plain string, and twelve clients would have been handed a parameter that can be null with no
/// sign of it anywhere. So the same census runs here, and a keyword this cannot answer for stops
/// the read.
fn parameter_type(data: &SpecParameterData) -> Result<String, Error> {
    let ParameterSchemaOrContent::Schema(schema) = &data.format else {
        return Ok(DEFAULT_PARAMETER_TYPE.to_owned());
    };

    let schema = serde_json::to_value(schema).map_err(|err| Error::UnserializableSchema {
        subject: preview(&data.name),
        reason: err.to_string(),
    })?;

    let members = schema.as_object().ok_or_else(|| Error::UnreadableSchema {
        subject: preview(&data.name),
    })?;
    for keyword in members.keys() {
        let modelled = MODELLED_KEYWORDS.contains(&keyword.as_str())
            || IGNORED_KEYWORDS.contains(&keyword.as_str());
        if !modelled {
            return Err(Error::UnmodelledSchemaKeyword {
                subject: preview(&data.name),
                keyword: preview(keyword),
            });
        }
    }

    Ok(members
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_PARAMETER_TYPE)
        .to_owned())
}

/// Follows `$ref` hops until an item is reached, which is also what stops a reference cycle.
fn resolve<'a, T>(
    entry: &'a ReferenceOr<T>,
    prefix: &str,
    lookup: impl Fn(&str) -> Option<&'a ReferenceOr<T>>,
    limits: &Limits,
) -> Result<&'a T, Error> {
    resolve_named(entry, prefix, lookup, limits).map(|(item, _)| item)
}

/// Same, also reporting the component the last hop went through, absent when there was no hop.
fn resolve_named<'a, T>(
    entry: &'a ReferenceOr<T>,
    prefix: &str,
    lookup: impl Fn(&str) -> Option<&'a ReferenceOr<T>>,
    limits: &Limits,
) -> Result<(&'a T, Option<String>), Error> {
    let mut current = entry;
    let mut through = None;
    let mut hops = 0;

    loop {
        match current {
            ReferenceOr::Item(item) => return Ok((item, through)),
            ReferenceOr::Reference { reference } => {
                if hops >= limits.max_reference_depth {
                    return Err(Error::ReferenceTooDeep {
                        reference: preview(reference),
                        limit: limits.max_reference_depth,
                    });
                }
                hops += 1;

                let name = reference
                    .strip_prefix(prefix)
                    .map(decode_pointer_segment)
                    .ok_or_else(|| Error::UnresolvableReference {
                        reference: preview(reference),
                    })?;
                current = lookup(&name).ok_or_else(|| Error::UnresolvableReference {
                    reference: preview(reference),
                })?;
                through = Some(name);
            }
        }
    }
}

/// Turns a JSON pointer segment back into the key it names.
pub(crate) fn decode_pointer_segment(segment: &str) -> String {
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

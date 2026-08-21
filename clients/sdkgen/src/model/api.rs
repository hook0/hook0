//! Everything a typed client needs, in terms no single language owns.
//!
//! [`EntityModel`] answers what the API *does*; this answers what it *carries*: the types of every
//! body it reads and writes, the contract its failures follow, and the credentials it accepts.
//! Targets read this and nothing else, so a fact about the API surface is established once, here,
//! rather than once per language.

use std::collections::BTreeMap;

use crate::error::{Error, preview};
use crate::limits::Limits;
use crate::model::EntityModel;
use crate::model::errors::ErrorModel;
use crate::model::security::SecurityModel;
use crate::model::shape::{ObjectShape, Shape, Shapes, derived_name, type_name};
use crate::snapshot::{Operation, RequestBody, ResponseBody, ResponseStatus, Snapshot};

/// Lowest status an operation answers a success with.
const SUCCESS_STATUS: u16 = 200;

/// Lowest status that is no longer a success.
const REDIRECTION_STATUS: u16 = 300;

/// Member an operation reads its body under, when that body is written where it is declared.
const REQUEST_MEMBER: &str = "request";

/// Member an operation writes its body under, when that body is written where it is declared.
const RESPONSE_MEMBER: &str = "response";

/// The whole API, as the targets are generated from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiModel {
    pub title: String,
    pub version: String,
    pub servers: Vec<String>,
    pub entities: EntityModel,
    /// Every object type a target declares, under the name it declares it with: the ones
    /// `components.schemas` names, and the ones derived for schemas written where they are used.
    ///
    /// Operations the `entity.verb` convention could not place carry no method, so no target
    /// renders them and the types of their bodies are not collected here either.
    pub schemas: BTreeMap<String, ObjectShape>,
    pub errors: ErrorModel,
    pub security: SecurityModel,
}

impl ApiModel {
    /// Every closed list of strings the model carries, under the name it is declared with.
    ///
    /// A target declares one type per entry, so the answer has to be the same for all of them:
    /// asked here rather than once per language, two targets cannot disagree on what the API
    /// declares. A list reached twice under one name has to spell the same values both times, since
    /// two enumerations sharing a name would be one type and whichever one lost would be a silent
    /// mis-decoding.
    pub fn enumerations(&self, limits: &Limits) -> Result<BTreeMap<String, Vec<String>>, Error> {
        let mut found = BTreeMap::new();

        for object in self.schemas.values() {
            for field in &object.fields {
                collect_enums(&field.shape, &mut found, limits, 0)?;
            }
        }
        for entity in self.entities.entities() {
            for method in &entity.methods {
                if let Some(shape) = method.request.as_ref() {
                    collect_enums(shape, &mut found, limits, 0)?;
                }
                if let Some((_, Some(shape))) = method.success.as_ref() {
                    collect_enums(shape, &mut found, limits, 0)?;
                }
            }
        }

        Ok(found)
    }

    /// Reads the whole model out of a snapshot.
    pub fn from_snapshot(snapshot: &Snapshot, limits: &Limits) -> Result<Self, Error> {
        let mut entities = EntityModel::from_snapshot(snapshot, limits)?;
        let mut shapes = Shapes::new(limits);

        for (name, schema) in snapshot.schemas() {
            shapes.read_component(name, schema)?;
        }

        for method in entities.methods_mut() {
            method.request = request_shape(&method.operation, &mut shapes, limits)?;
            method.success = success_shape(&method.operation, &mut shapes, limits)?;
        }

        let schemas = shapes.into_objects();
        let errors = ErrorModel::discover(snapshot.operations(), &schemas)?;
        let security = SecurityModel::read(snapshot)?;

        Ok(Self {
            title: snapshot.title().to_owned(),
            version: snapshot.version().to_owned(),
            servers: snapshot.servers().to_vec(),
            entities,
            schemas,
            errors,
            security,
        })
    }
}

/// Gathers every closed list of strings a shape carries, however deeply it sits.
fn collect_enums(
    shape: &Shape,
    found: &mut BTreeMap<String, Vec<String>>,
    limits: &Limits,
    depth: usize,
) -> Result<(), Error> {
    if depth > limits.max_shape_depth {
        return Err(Error::SchemaTooDeep {
            subject: preview("a field of the model"),
            limit: limits.max_shape_depth,
        });
    }

    match shape {
        Shape::Enum { name, values } => match found.get(name) {
            Some(first) if first != values => Err(Error::SchemaNameCollision {
                name: preview(name),
                first: preview(&first.join(", ")),
                second: preview(&values.join(", ")),
            }),
            Some(_) => Ok(()),
            None => {
                found.insert(name.clone(), values.clone());
                Ok(())
            }
        },
        Shape::Array(inner) | Shape::Map(inner) => collect_enums(inner, found, limits, depth + 1),
        Shape::Object(object) => {
            for field in &object.fields {
                collect_enums(&field.shape, found, limits, depth + 1)?;
            }
            Ok(())
        }
        Shape::Scalar(_) | Shape::Named(_) | Shape::Json => Ok(()),
    }
}

/// The type an operation reads.
///
/// A body reached through a component is that component; one written where the operation declares
/// it is declared under a name derived from the operation, so a language that has to name the
/// argument type of a method has one.
fn request_shape(
    operation: &Operation,
    shapes: &mut Shapes<'_>,
    limits: &Limits,
) -> Result<Option<Shape>, Error> {
    let Some(RequestBody::Json(schema)) = operation.request_body.as_ref() else {
        return Ok(None);
    };

    // The snapshot follows the reference to give a tool caller the fields it has to fill in, which
    // is what loses the name; the operation carries it separately for the targets that need it.
    if let Some(named) = operation.request_body_schema.as_ref() {
        return Ok(Some(Shape::Named(named.clone())));
    }

    let (name, origin) = derived(operation, REQUEST_MEMBER, limits)?;
    Ok(Some(shapes.read(schema, &name, &origin)?))
}

/// The status an operation answers on success and the type it writes back, if any.
///
/// The lowest success is the one a client returns: a method whose return type changed with the
/// status the API happened to pick would be unusable, and the document says nothing about when one
/// success is answered rather than another.
fn success_shape(
    operation: &Operation,
    shapes: &mut Shapes<'_>,
    limits: &Limits,
) -> Result<Option<(u16, Option<Shape>)>, Error> {
    let success = operation.responses.iter().find_map(|response| {
        let ResponseStatus::Code(status) = response.status else {
            return None;
        };
        (SUCCESS_STATUS..REDIRECTION_STATUS)
            .contains(&status)
            .then_some((status, response.body.as_ref()))
    });

    let Some((status, body)) = success else {
        return Ok(None);
    };

    let Some(ResponseBody::Json(schema)) = body else {
        return Ok(Some((status, None)));
    };

    // A schema pointing at a component names it outright, so the derived name goes unused; only a
    // body written where the operation declares it claims one.
    let (name, origin) = derived(operation, RESPONSE_MEMBER, limits)?;
    Ok(Some((status, Some(shapes.read(schema, &name, &origin)?))))
}

/// The name a schema of that member is declared under, and where a message says it came from.
fn derived(
    operation: &Operation,
    member: &str,
    limits: &Limits,
) -> Result<(String, String), Error> {
    let subject = operation
        .operation_id
        .clone()
        .unwrap_or_else(|| operation.location());

    let name = derived_name(&type_name(&subject), member, limits)?;
    Ok((name, format!("{subject} {member}")))
}

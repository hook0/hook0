pub mod api;
pub mod errors;
pub mod security;
pub mod shape;

use std::collections::BTreeMap;

use crate::error::{Error, preview};
use crate::limits::Limits;
use crate::snapshot::{Operation, Snapshot};

pub use api::ApiModel;
pub use errors::{ErrorModel, ProblemCatalogue};
pub use security::{Scheme, SecurityModel};
pub use shape::{Field, IGNORED_KEYWORDS, MODELLED_KEYWORDS, ObjectShape, Scalar, Shape};

/// Separator between the entity and the verb of an operation id.
const VERB_SEPARATOR: char = '.';

/// What an entity method does, once its verb has been read.
///
/// Verbs outside the canonical vocabulary stay as [`Verb::Named`]: they become a method of their
/// entity under the name the spec gives them, and are never dropped.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verb {
    List,
    Get,
    Create,
    Update,
    Delete,
    Named(String),
}

impl Verb {
    fn read(verb: &str) -> Self {
        match verb {
            "list" => Self::List,
            "get" | "load" => Self::Get,
            "create" => Self::Create,
            "update" => Self::Update,
            "delete" | "remove" => Self::Delete,
            named => Self::Named(named.to_owned()),
        }
    }

    /// Whether the verb belongs to the vocabulary every target renders the same way.
    pub fn is_canonical(&self) -> bool {
        !matches!(self, Self::Named(_))
    }
}

/// One operation of an entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    /// Operation id the spec declares, `entity.verb`.
    pub operation_id: String,
    /// Verb as the spec writes it, before the canonical vocabulary is applied.
    pub verb_text: String,
    pub verb: Verb,
    pub operation: Operation,
    /// Type the method reads, absent when it reads no body.
    ///
    /// Shapes point into the schema registry an [`ApiModel`] carries, so they are read there:
    /// an entity model built on its own describes what the API does without naming any type, and
    /// leaves this absent.
    pub request: Option<Shape>,
    /// Status the method answers on success and the type it writes back, both absent under the
    /// same condition as [`Method::request`].
    pub success: Option<(u16, Option<Shape>)>,
}

/// A group of operations sharing the leading segment of their operation id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub name: String,
    pub methods: Vec<Method>,
}

/// Why an operation could not be attached to an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nonconformity {
    /// The spec declares no `operationId`.
    MissingOperationId,
    /// The operation id carries no `entity.verb` separator.
    MissingVerbSeparator,
    /// The operation id starts with the separator, leaving no entity to attach to.
    EmptyEntityName,
    /// The operation id ends with the separator, leaving no verb to name a method with.
    EmptyVerb,
}

/// An operation the `entity.verb` convention cannot place, kept so it is never lost in silence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnconventionalOperation {
    pub operation: Operation,
    pub reason: Nonconformity,
}

/// Entities and their methods, derived from operation ids alone.
///
/// Every operation of the snapshot lands either on an entity method or in
/// [`EntityModel::unconventional`], never nowhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityModel {
    entities: Vec<Entity>,
    unconventional: Vec<UnconventionalOperation>,
}

impl EntityModel {
    /// Groups the operations of a snapshot by the entity their operation id names.
    pub fn from_snapshot(snapshot: &Snapshot, limits: &Limits) -> Result<Self, Error> {
        let mut grouped: BTreeMap<String, Vec<Method>> = BTreeMap::new();
        let mut unconventional = Vec::new();

        for operation in snapshot.operations() {
            match split_operation_id(operation) {
                Err(reason) => unconventional.push(UnconventionalOperation {
                    operation: operation.clone(),
                    reason,
                }),
                Ok((entity, verb_text)) => {
                    let methods = grouped.entry(entity.clone()).or_default();

                    if methods.len() == limits.max_methods_per_entity {
                        return Err(Error::TooManyMethods {
                            entity: preview(&entity),
                            count: methods.len() + 1,
                            limit: limits.max_methods_per_entity,
                        });
                    }

                    methods.push(Method {
                        operation_id: format!("{entity}{VERB_SEPARATOR}{verb_text}"),
                        verb: Verb::read(&verb_text),
                        verb_text,
                        operation: operation.clone(),
                        request: None,
                        success: None,
                    });
                }
            }
        }

        if grouped.len() > limits.max_entities {
            return Err(Error::TooManyEntities {
                count: grouped.len(),
                limit: limits.max_entities,
            });
        }

        let entities = grouped
            .into_iter()
            .map(|(name, mut methods)| {
                methods.sort_by(|left, right| left.verb_text.cmp(&right.verb_text));
                Entity { name, methods }
            })
            .collect();

        unconventional.sort_by_key(|set_aside| set_aside.operation.location());

        Ok(Self {
            entities,
            unconventional,
        })
    }

    /// Entities, ordered by name.
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Operations the `entity.verb` convention cannot place.
    pub fn unconventional(&self) -> &[UnconventionalOperation] {
        &self.unconventional
    }

    /// How many methods every entity carries, taken together.
    pub fn method_count(&self) -> usize {
        self.entities
            .iter()
            .map(|entity| entity.methods.len())
            .sum()
    }

    /// The entity carrying that name.
    pub fn entity(&self, name: &str) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.name == name)
    }

    /// Every method of every entity, for the reader that fills in the types they carry.
    pub(crate) fn methods_mut(&mut self) -> impl Iterator<Item = &mut Method> {
        self.entities
            .iter_mut()
            .flat_map(|entity| entity.methods.iter_mut())
    }
}

impl Entity {
    /// The method reached by that verb, as the spec writes it.
    pub fn method(&self, verb_text: &str) -> Option<&Method> {
        self.methods
            .iter()
            .find(|method| method.verb_text == verb_text)
    }
}

/// Reads the entity and the verb out of an operation id, the leading segment naming the entity.
fn split_operation_id(operation: &Operation) -> Result<(String, String), Nonconformity> {
    let operation_id = operation
        .operation_id
        .as_ref()
        .ok_or(Nonconformity::MissingOperationId)?;

    let (entity, verb) = operation_id
        .split_once(VERB_SEPARATOR)
        .ok_or(Nonconformity::MissingVerbSeparator)?;

    if entity.is_empty() {
        return Err(Nonconformity::EmptyEntityName);
    }
    if verb.is_empty() {
        return Err(Nonconformity::EmptyVerb);
    }

    Ok((entity.to_owned(), verb.to_owned()))
}

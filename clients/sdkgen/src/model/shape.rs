//! The type language every target is generated from.
//!
//! A [`Shape`] says what a value is without saying how any language writes it: the targets decide
//! that. Reading one out of a JSON schema is deliberately strict — a keyword the language does not
//! model stops the read rather than being skipped, because a keyword skipped here is a keyword
//! skipped in every generated client at once. `nullable` is the cautionary case: ignoring it would
//! yield ten clients that declare a field non-optional and crash the day the API sends a null.
//!
//! Optionality is therefore membership in `required`, and nothing else.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::error::{Error, preview};
use crate::limits::Limits;
use crate::snapshot::{SCHEMA_REFERENCE_PREFIX, decode_pointer_segment};

/// Schema keywords the reader turns into a shape.
///
/// Together with [`IGNORED_KEYWORDS`] this is the whole vocabulary the reader accepts: anything
/// else stops the read by name, so a keyword appearing in a later snapshot is reported rather than
/// silently dropped.
pub const MODELLED_KEYWORDS: [&str; 9] = [
    "$ref",
    "additionalProperties",
    "description",
    "enum",
    "format",
    "items",
    "properties",
    "required",
    "type",
];

/// Schema keywords the reader knowingly leaves out: they document a schema or illustrate it
/// without changing the type a target declares for it.
pub const IGNORED_KEYWORDS: [&str; 1] = ["example"];

/// A value that no target breaks down any further.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scalar {
    String,
    Uuid,
    DateTime,
    Date,
    Url,
    Integer32,
    Integer64,
    Number,
    Boolean,
}

/// What a value is, in terms no single language owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    Scalar(Scalar),
    Array(Box<Shape>),
    /// An object whose keys are open, each value shaped alike.
    Map(Box<Shape>),
    /// A closed list of strings, under the name a target declares it with.
    Enum {
        name: String,
        values: Vec<String>,
    },
    /// A reference to a schema declared elsewhere, by name.
    Named(String),
    /// An object the document writes where it is used, under the name derived for it. The same
    /// object is also registered in [`crate::ApiModel::schemas`], so a target that declares its
    /// types up front and one that renders them where they are used both have what they need.
    Object(Box<ObjectShape>),
    /// A value whose shape the document does not describe.
    Json,
}

/// One member of an object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub shape: Shape,
    /// Whether the document lists the field in `required`, which is the only thing that makes a
    /// field mandatory.
    pub required: bool,
    pub description: Option<String>,
}

/// An object type, under the name targets declare it with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectShape {
    pub name: String,
    /// Fields, ordered by name so the same document always yields the same declaration.
    pub fields: Vec<Field>,
}

/// Where the schemas of a document are named from, which error messages point back at.
const COMPONENT_ORIGIN: &str = "components.schemas";

/// Reads schemas into shapes, collecting every object type it meets along the way.
///
/// The names of component schemas and the names derived for inline ones share one namespace: two
/// objects reaching the same name would be one type in a generated client, silently merging two
/// unrelated shapes, so a collision stops the read.
pub(crate) struct Shapes<'a> {
    limits: &'a Limits,
    objects: BTreeMap<String, ObjectShape>,
    origins: BTreeMap<String, String>,
}

impl<'a> Shapes<'a> {
    pub(crate) fn new(limits: &'a Limits) -> Self {
        Self {
            limits,
            objects: BTreeMap::new(),
            origins: BTreeMap::new(),
        }
    }

    /// Reads a schema `components.schemas` declares, under the name it declares it with.
    pub(crate) fn read_component(&mut self, name: &str, schema: &Value) -> Result<(), Error> {
        let origin = format!("{COMPONENT_ORIGIN}.{name}");
        match self.read(schema, name, &origin)? {
            Shape::Object(_) => Ok(()),
            _ => Err(Error::NonObjectSchema {
                schema: preview(name),
            }),
        }
    }

    /// Reads a schema written where it is used, under the name derived for it.
    pub(crate) fn read(
        &mut self,
        schema: &Value,
        name: &str,
        origin: &str,
    ) -> Result<Shape, Error> {
        self.read_at(schema, name, origin, 0)
    }

    /// Every object type the read met, component and derived alike.
    pub(crate) fn into_objects(self) -> BTreeMap<String, ObjectShape> {
        self.objects
    }

    fn read_at(
        &mut self,
        schema: &Value,
        name: &str,
        origin: &str,
        depth: usize,
    ) -> Result<Shape, Error> {
        if depth > self.limits.max_shape_depth {
            return Err(Error::SchemaTooDeep {
                subject: preview(origin),
                limit: self.limits.max_shape_depth,
            });
        }

        let members = schema.as_object().ok_or_else(|| Error::UnreadableSchema {
            subject: preview(origin),
        })?;

        for keyword in members.keys() {
            let modelled = MODELLED_KEYWORDS.contains(&keyword.as_str())
                || IGNORED_KEYWORDS.contains(&keyword.as_str());
            if !modelled {
                return Err(Error::UnmodelledSchemaKeyword {
                    subject: preview(origin),
                    keyword: preview(keyword),
                });
            }
        }

        if let Some(reference) = members.get("$ref") {
            return read_reference(reference);
        }
        if let Some(values) = members.get("enum") {
            return self.read_enum(values, members.get("type"), name, origin);
        }

        match members.get("type") {
            None if members.contains_key("properties")
                || members.contains_key("additionalProperties") =>
            {
                self.read_object(members, name, origin, depth)
            }
            None => Err(Error::UntypedSchema {
                subject: preview(origin),
            }),
            Some(Value::String(declared)) => match declared.as_str() {
                "object" => self.read_object(members, name, origin, depth),
                "array" => self.read_array(members, name, origin, depth),
                "string" => Ok(Shape::Scalar(string_scalar(members.get("format")))),
                "integer" => Ok(Shape::Scalar(integer_scalar(members.get("format")))),
                "number" => Ok(Shape::Scalar(Scalar::Number)),
                "boolean" => Ok(Shape::Scalar(Scalar::Boolean)),
                declared => Err(Error::UnknownSchemaType {
                    subject: preview(origin),
                    declared: preview(declared),
                }),
            },
            Some(declared) => Err(Error::UnknownSchemaType {
                subject: preview(origin),
                declared: preview(&declared.to_string()),
            }),
        }
    }

    fn read_enum(
        &mut self,
        values: &Value,
        declared: Option<&Value>,
        name: &str,
        origin: &str,
    ) -> Result<Shape, Error> {
        let is_string = match declared {
            None => true,
            Some(Value::String(declared)) => declared == "string",
            Some(_) => false,
        };
        let Some(values) = values.as_array().filter(|_| is_string) else {
            return Err(Error::UnmodelledEnum {
                subject: preview(origin),
            });
        };

        if values.len() > self.limits.max_enum_values {
            return Err(Error::TooManyEnumValues {
                subject: preview(origin),
                count: values.len(),
                limit: self.limits.max_enum_values,
            });
        }

        let mut read = Vec::with_capacity(values.len());
        for value in values {
            let value = value.as_str().ok_or_else(|| Error::UnmodelledEnum {
                subject: preview(origin),
            })?;
            read.push(value.to_owned());
        }

        if read.is_empty() {
            return Err(Error::UnmodelledEnum {
                subject: preview(origin),
            });
        }

        let name = self.claim(name, origin)?;
        Ok(Shape::Enum { name, values: read })
    }

    fn read_array(
        &mut self,
        members: &serde_json::Map<String, Value>,
        name: &str,
        origin: &str,
        depth: usize,
    ) -> Result<Shape, Error> {
        // An array whose items the document leaves undescribed carries values no target can name,
        // which is what `Json` stands for.
        let Some(items) = members.get("items") else {
            return Ok(Shape::Array(Box::new(Shape::Json)));
        };

        let items = self.read_at(items, name, origin, depth + 1)?;
        Ok(Shape::Array(Box::new(items)))
    }

    fn read_object(
        &mut self,
        members: &serde_json::Map<String, Value>,
        name: &str,
        origin: &str,
        depth: usize,
    ) -> Result<Shape, Error> {
        let properties = members.get("properties");
        let open = members.get("additionalProperties");

        match (properties, open) {
            (Some(_), Some(_)) => Err(Error::AmbiguousObjectSchema {
                subject: preview(origin),
            }),
            (None, None) => Ok(Shape::Json),
            // `additionalProperties` written as a bare boolean says whether unknown keys are
            // allowed, not what they hold, which leaves the values undescribed.
            (None, Some(open)) if !open.is_object() => Ok(Shape::Json),
            (None, Some(open)) => {
                let values = self.read_at(open, name, origin, depth + 1)?;
                Ok(Shape::Map(Box::new(values)))
            }
            (Some(properties), None) => {
                let object =
                    self.read_fields(properties, members.get("required"), name, origin, depth)?;
                self.register(object.clone(), origin)?;
                Ok(Shape::Object(Box::new(object)))
            }
        }
    }

    fn read_fields(
        &mut self,
        properties: &Value,
        required: Option<&Value>,
        name: &str,
        origin: &str,
        depth: usize,
    ) -> Result<ObjectShape, Error> {
        let properties = properties
            .as_object()
            .ok_or_else(|| Error::UnreadableSchema {
                subject: preview(origin),
            })?;

        if properties.len() > self.limits.max_fields_per_object {
            return Err(Error::TooManyFields {
                object: preview(name),
                count: properties.len(),
                limit: self.limits.max_fields_per_object,
            });
        }

        let required = read_required(required, origin)?;

        let mut fields = Vec::with_capacity(properties.len());
        for (field, schema) in properties {
            let member_name = derived_name(name, field, self.limits)?;
            let member_origin = format!("{origin}.{field}");
            let shape = self.read_at(schema, &member_name, &member_origin, depth + 1)?;

            fields.push(Field {
                name: field.clone(),
                shape,
                required: required.iter().any(|entry| entry == field),
                description: schema
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            });
        }

        fields.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(ObjectShape {
            name: name.to_owned(),
            fields,
        })
    }

    fn register(&mut self, object: ObjectShape, origin: &str) -> Result<(), Error> {
        self.claim(&object.name, origin)?;

        if self.objects.len() == self.limits.max_schemas {
            return Err(Error::TooManySchemas {
                count: self.objects.len() + 1,
                limit: self.limits.max_schemas,
            });
        }

        self.objects.insert(object.name.clone(), object);
        Ok(())
    }

    /// Reserves a type name, reporting where it was already taken from.
    fn claim(&mut self, name: &str, origin: &str) -> Result<String, Error> {
        if let Some(first) = self.origins.get(name) {
            return Err(Error::SchemaNameCollision {
                name: preview(name),
                first: preview(first),
                second: preview(origin),
            });
        }

        self.origins.insert(name.to_owned(), origin.to_owned());
        Ok(name.to_owned())
    }
}

/// The component a schema points at, absent when it points at nothing or describes itself.
pub(crate) fn referenced_name(schema: &Value) -> Option<String> {
    schema
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| reference.strip_prefix(SCHEMA_REFERENCE_PREFIX))
        .map(decode_pointer_segment)
}

/// A reference reaching outside `components.schemas` names nothing a target could declare.
fn read_reference(reference: &Value) -> Result<Shape, Error> {
    reference
        .as_str()
        .and_then(|reference| reference.strip_prefix(SCHEMA_REFERENCE_PREFIX))
        .map(decode_pointer_segment)
        .map(Shape::Named)
        .ok_or_else(|| Error::UnresolvableReference {
            reference: preview(&reference.to_string()),
        })
}

fn read_required(required: Option<&Value>, origin: &str) -> Result<Vec<String>, Error> {
    let Some(required) = required else {
        return Ok(Vec::new());
    };

    let required = required.as_array().ok_or_else(|| Error::UnreadableSchema {
        subject: preview(origin),
    })?;

    required
        .iter()
        .map(|field| {
            field
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| Error::UnreadableSchema {
                    subject: preview(origin),
                })
        })
        .collect()
}

/// A string carries a scalar of its own when it names a format every target has a type for, and
/// stays a plain string otherwise rather than being refused.
fn string_scalar(format: Option<&Value>) -> Scalar {
    match format.and_then(Value::as_str) {
        Some("uuid") => Scalar::Uuid,
        Some("date-time") => Scalar::DateTime,
        Some("date") => Scalar::Date,
        Some("url") => Scalar::Url,
        _ => Scalar::String,
    }
}

/// An integer of unstated width is read as the wider one: a target narrowing it would truncate the
/// values the API already sends.
fn integer_scalar(format: Option<&Value>) -> Scalar {
    match format.and_then(Value::as_str) {
        Some("int32") => Scalar::Integer32,
        _ => Scalar::Integer64,
    }
}

/// The name an inline schema is declared under: what owns it, then the member it sits in, so
/// `Subscription.target` is declared as `SubscriptionTarget`.
pub(crate) fn derived_name(owner: &str, member: &str, limits: &Limits) -> Result<String, Error> {
    let name = format!("{owner}{}", type_name(member));

    if name.len() > limits.max_identifier_bytes {
        return Err(Error::IdentifierTooLong {
            identifier: preview(&name),
            size: name.len(),
            limit: limits.max_identifier_bytes,
        });
    }

    Ok(name)
}

/// Turns text the snapshot writes into the fragment a type name is built from.
pub(crate) fn type_name(text: &str) -> String {
    let mut name = String::with_capacity(text.len());

    for segment in text.split(|character: char| !character.is_alphanumeric()) {
        let mut characters = segment.chars();
        let Some(first) = characters.next() else {
            continue;
        };
        name.extend(first.to_uppercase());
        name.push_str(characters.as_str());
    }

    // Text made of nothing a type name can be built from still has to name something distinct,
    // and a collision on what comes out is reported rather than merged.
    if name.is_empty() {
        return text.to_owned();
    }

    name
}

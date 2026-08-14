//! Emits the generated half of the Python SDK.
//!
//! The package is published to PyPI with no copy of the OpenAPI snapshot beside it, so the types,
//! the problems and the request layer travel as committed source rather than as a build artefact.
//! Everything the API declares — one class per named schema, one enumeration per closed list of
//! strings, one exception per problem the error contract can report, one method per operation — is
//! written here; everything the API does not declare — how a request reaches the network, how a
//! send is retried, how a webhook signature is verified — is hand-written beside this directory and
//! never regenerated.
//!
//! The two halves meet at two seams and nowhere else. The generated code reads its decoding
//! helpers from the hand-written runtime module above it, and it calls whatever object it is handed
//! as a transport: nothing here knows what a socket is, and nothing beside it knows what the API
//! declares.
//!
//! Anything the emitter cannot make sense of — a parameter travelling somewhere a client cannot put
//! it, a type name two declarations would answer to, a scalar no annotation covers — stops the
//! emission rather than yielding a smaller SDK.

use std::collections::{BTreeMap, BTreeSet};

use crate::emit::{EmittedFile, FileTree, Ownership, RelativePath, banner};
use crate::error::{Error, preview};
use crate::identifier::{Case, checked_words, escape, render};
use crate::limits::Limits;
use crate::model::{ApiModel, Entity, ErrorModel, Field, Method, ObjectShape, Scalar, Shape};
use crate::snapshot::{PUBLIC_TAG, Parameter, ParameterLocation};
use crate::targets::{LanguageSpec, ScalarNames, Target, update_command};

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "python";

/// Where the generated half of the package lands.
///
/// Everything under it belongs to the generator; everything beside it is hand-written.
const ROOT: &str = "clients/python/src/hook0/generated";

/// Module the generated code reads its decoding helpers from, one directory above the generated
/// package. Written as a relative import, so it names a neighbour rather than a package.
const RUNTIME_MODULE: &str = "..runtime";

/// The modules this target writes, each one holding one layer of the surface.
const MODELS_MODULE: &str = "models";
const ERRORS_MODULE: &str = "errors";
const SYNC_MODULE: &str = "api";
const ASYNC_MODULE: &str = "aio";

/// Suffix telling an operation group from a type of the same name: the document names an entity
/// and a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "Api";

/// Suffix the asynchronous flavour of an operation group carries.
const ASYNC_GROUP_SUFFIX: &str = "AsyncApi";

/// Suffix an exception carries, so a problem and a type spelling the same word stay apart.
const EXCEPTION_SUFFIX: &str = "Error";

/// Longest fragment of a snapshot description a docstring carries.
const MAX_DOCSTRING_CHARS: usize = 200;

/// Lowest status a response is read as a success under.
const LOWEST_SUCCESS: u16 = 200;

/// Lowest status that is no longer a success.
const LOWEST_REDIRECTION: u16 = 300;

/// This target, as the registry carries it.
pub(super) fn target() -> Target {
    Target {
        name: NAME,
        tag: PUBLIC_TAG,
        root: ROOT,
        // The whole directory is generated, so a type the document stopped declaring takes its file
        // with it instead of lingering as an orphan nothing imports.
        ownership: Ownership::Directory,
        language: super::python(),
        emit,
    }
}

/// Everything the generated half of the package is made of.
fn emit(language: &LanguageSpec, model: &ApiModel) -> Result<FileTree, Error> {
    let limits = Limits::DEFAULT;
    let banner = banner(language.comment, &update_command(NAME), &limits)?;

    let enums = enumerations(model, &limits)?;
    let types = Types::read(model, &enums, language, &limits)?;

    let files = vec![
        file(
            MODELS_MODULE,
            &banner,
            &models(model, &enums, &types, language, &limits)?,
            &limits,
        )?,
        file(
            ERRORS_MODULE,
            &banner,
            &errors(model, &types, language, &limits)?,
            &limits,
        )?,
        file(
            SYNC_MODULE,
            &banner,
            &requests(model, &types, language, &limits, Flavour::Sync)?,
            &limits,
        )?,
        file(
            ASYNC_MODULE,
            &banner,
            &requests(model, &types, language, &limits, Flavour::Async)?,
            &limits,
        )?,
        file("__init__", &banner, &package(&types), &limits)?,
    ];

    FileTree::build(files, &limits)
}

fn file(module: &str, banner: &str, body: &str, limits: &Limits) -> Result<EmittedFile, Error> {
    Ok(EmittedFile {
        path: RelativePath::build(&format!("{module}.py"), limits)?,
        contents: format!("{banner}{body}"),
    })
}

/// Whether a request layer waits for its transport or calls it outright.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flavour {
    Sync,
    Async,
}

impl Flavour {
    fn keyword(self) -> &'static str {
        match self {
            Self::Sync => "def",
            Self::Async => "async def",
        }
    }

    fn call(self) -> &'static str {
        match self {
            Self::Sync => "self._transport.request(",
            Self::Async => "await self._transport.request(",
        }
    }
}

/// Every name the generated package declares, under the layer that declares it.
///
/// Names are settled before a single line is written, so a collision is reported as a collision
/// rather than as a class that silently replaces another one when the module is imported.
struct Types {
    /// Type each named schema is declared under, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Type each closed list of strings is declared under.
    enums: BTreeMap<String, String>,
    /// Exception each problem is raised as, by the value the catalogue lists.
    problems: BTreeMap<String, String>,
    /// Exception every problem is a kind of.
    problem_base: String,
    /// Enumeration the discriminant of the error contract carries.
    problem_enum: String,
    /// Operation groups, by entity name, for each flavour.
    groups: BTreeMap<String, (String, String)>,
    /// Everything the package exports, sorted.
    exported: BTreeSet<String>,
    /// What the language calls each scalar the model carries.
    scalars: ScalarNames,
}

impl Types {
    fn read(
        model: &ApiModel,
        enums: &BTreeMap<String, Vec<String>>,
        language: &LanguageSpec,
        limits: &Limits,
    ) -> Result<Self, Error> {
        let mut claimed: BTreeMap<String, String> = BTreeMap::new();
        let mut claim = |name: String, origin: &str| -> Result<String, Error> {
            if let Some(first) = claimed.get(&name) {
                return Err(Error::SchemaNameCollision {
                    name: preview(&name),
                    first: preview(first),
                    second: preview(origin),
                });
            }
            claimed.insert(name.clone(), origin.to_owned());
            Ok(name)
        };

        let mut schemas = BTreeMap::new();
        for name in model.schemas.keys() {
            let declared = claim(ident(name, Case::UpperCamel, language, limits)?, name)?;
            schemas.insert(name.clone(), declared);
        }

        let mut declared_enums = BTreeMap::new();
        for name in enums.keys() {
            let declared = claim(ident(name, Case::UpperCamel, language, limits)?, name)?;
            declared_enums.insert(name.clone(), declared);
        }

        let problem_enum = enum_of(&model.errors, &declared_enums)?;

        let base = format!(
            "{}{EXCEPTION_SUFFIX}",
            ident(&model.errors.schema, Case::UpperCamel, language, limits)?
        );
        let problem_base = claim(base, &model.errors.schema)?;

        let mut problems = BTreeMap::new();
        for value in model.errors.catalogue.values() {
            let name = format!(
                "{}{EXCEPTION_SUFFIX}",
                ident(value, Case::UpperCamel, language, limits)?
            );
            problems.insert(value.clone(), claim(name, value)?);
        }

        let mut groups = BTreeMap::new();
        for entity in model.entities.entities() {
            let stem = ident(&entity.name, Case::UpperCamel, language, limits)?;
            let blocking = claim(format!("{stem}{GROUP_SUFFIX}"), &entity.name)?;
            let waiting = claim(format!("{stem}{ASYNC_GROUP_SUFFIX}"), &entity.name)?;
            groups.insert(entity.name.clone(), (blocking, waiting));
        }

        let exported = claimed.keys().cloned().collect();

        Ok(Self {
            schemas,
            enums: declared_enums,
            problems,
            problem_base,
            problem_enum,
            groups,
            exported,
            scalars: language.scalars,
        })
    }

    fn schema(&self, name: &str) -> Result<&str, Error> {
        self.schemas
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| Error::UnresolvableReference {
                reference: preview(name),
            })
    }

    fn enumeration(&self, name: &str) -> Result<&str, Error> {
        self.enums
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| Error::UnresolvableReference {
                reference: preview(name),
            })
    }

    fn group(&self, entity: &str, flavour: Flavour) -> Result<&str, Error> {
        self.groups
            .get(entity)
            .map(|(blocking, waiting)| match flavour {
                Flavour::Sync => blocking.as_str(),
                Flavour::Async => waiting.as_str(),
            })
            .ok_or_else(|| Error::UnresolvableReference {
                reference: preview(entity),
            })
    }
}

/// The enumeration the discriminant of the error contract is read through.
///
/// The catalogue is the values of one closed list of strings the error schema declares, so the
/// enumeration already exists among the types: it is found rather than declared twice.
fn enum_of(errors: &ErrorModel, enums: &BTreeMap<String, String>) -> Result<String, Error> {
    let discriminant = errors
        .shape
        .fields
        .iter()
        .find(|field| field.name == errors.discriminant)
        .ok_or_else(|| Error::ErrorSchemaWithoutCatalogue {
            schema: preview(&errors.schema),
        })?;

    let Shape::Enum { name, .. } = &discriminant.shape else {
        return Err(Error::ErrorSchemaWithoutCatalogue {
            schema: preview(&errors.schema),
        });
    };

    enums
        .get(name)
        .cloned()
        .ok_or_else(|| Error::UnresolvableReference {
            reference: preview(name),
        })
}

/// Every closed list of strings the model carries, under the name it is declared with.
///
/// A list reached twice under one name has to spell the same values both times: two enumerations
/// sharing a name would be one class, and whichever one lost would be a silent mis-decoding.
fn enumerations(model: &ApiModel, limits: &Limits) -> Result<BTreeMap<String, Vec<String>>, Error> {
    let mut found = BTreeMap::new();

    for object in model.schemas.values() {
        for field in &object.fields {
            collect_enums(&field.shape, &mut found, limits, 0)?;
        }
    }
    for entity in model.entities.entities() {
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

/// What a module imports, gathered while its body is written rather than guessed afterwards.
#[derive(Debug, Default)]
struct Needs {
    /// Standard-library modules an annotation names.
    modules: BTreeSet<&'static str>,
    /// Helpers read from the hand-written runtime module.
    runtime: BTreeSet<&'static str>,
    /// Names read from the layer below.
    below: BTreeSet<String>,
}

impl Needs {
    fn module(&mut self, name: &'static str) {
        self.modules.insert(name);
    }

    fn helper(&mut self, name: &'static str) -> &'static str {
        self.runtime.insert(name);
        name
    }

    fn declared(&mut self, name: &str) {
        self.below.insert(name.to_owned());
    }
}

/// The types every body the API reads and writes is made of.
fn models(
    model: &ApiModel,
    enums: &BTreeMap<String, Vec<String>>,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut needs = Needs::default();
    let mut body = String::new();

    for (name, values) in enums {
        body.push_str(&enumeration(
            types.enumeration(name)?,
            values,
            language,
            limits,
        )?);
    }
    for (name, object) in &model.schemas {
        body.push_str(&structure(
            types.schema(name)?,
            object,
            types,
            &mut needs,
            language,
            limits,
        )?);
    }

    // The standard-library imports are one block: straight imports first, then the ones naming
    // what they bring in, each set sorted, which is the one arrangement an import sorter leaves
    // alone.
    let mut header = String::from(
        "\"\"\"The types the API carries, one class per schema it declares.\"\"\"\n\n\
         from __future__ import annotations\n\n",
    );
    for module in &needs.modules {
        header.push_str(&format!("import {module}\n"));
    }
    header.push_str("from dataclasses import dataclass\n");
    if !enums.is_empty() {
        header.push_str("from enum import StrEnum\n");
    }
    header.push_str("from typing import Any\n\n");
    header.push_str(&runtime_import(&needs));

    Ok(format!("{header}{body}"))
}

fn runtime_import(needs: &Needs) -> String {
    if needs.runtime.is_empty() {
        return String::new();
    }

    let mut import = format!("from {RUNTIME_MODULE} import (\n");
    for helper in &needs.runtime {
        import.push_str(&format!("    {helper},\n"));
    }
    import.push_str(")\n");
    import
}

fn enumeration(
    declared: &str,
    values: &[String],
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut members: BTreeMap<String, &String> = BTreeMap::new();
    let mut source = format!(
        "\n\nclass {declared}(StrEnum):\n    \"\"\"One of the values the API answers with.\"\"\"\n\n"
    );

    for value in values {
        let member = ident(value, Case::ScreamingSnake, language, limits)?;
        if let Some(first) = members.insert(member.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(value),
            });
        }
        source.push_str(&format!("    {member} = {}\n", literal(value)));
    }

    Ok(source)
}

fn structure(
    declared: &str,
    object: &ObjectShape,
    types: &Types,
    needs: &mut Needs,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let ordered = ordered_fields(object);

    let mut source = format!("\n\n@dataclass(frozen=True)\nclass {declared}:\n");
    source.push_str(&format!(
        "    \"\"\"{}\"\"\"\n\n",
        docstring(&format!("The `{}` the API declares.", object.name))
    ));

    for field in &ordered {
        let name = ident(&field.name, Case::Snake, language, limits)?;
        let annotation = annotation(&field.shape, types, needs, 0)?;
        if field.required {
            source.push_str(&format!("    {name}: {annotation}\n"));
        } else {
            source.push_str(&format!("    {name}: {annotation} | None = None\n"));
        }
    }

    source.push_str(&format!(
        "\n    @classmethod\n    def from_json(cls, value: Any) -> {declared}:\n        \
         \"\"\"Read one out of what the API answered.\"\"\"\n        \
         fields = {}(value, {})\n        return cls(\n",
        needs.helper("as_fields"),
        literal(&object.name)
    ));
    for field in &ordered {
        let reader = reader(&field.shape, types, needs, 0)?;
        let helper = if field.required {
            needs.helper("read")
        } else {
            needs.helper("maybe")
        };
        source.push_str(&format!(
            "            {helper}(fields, {}, {reader}),\n",
            literal(&field.name)
        ));
    }
    source.push_str("        )\n");

    source.push_str(
        "\n    def to_json(self) -> dict[str, Any]:\n        \
         \"\"\"Write one back the way the API reads it.\"\"\"\n        \
         out: dict[str, Any] = {}\n",
    );
    for field in &ordered {
        let name = ident(&field.name, Case::Snake, language, limits)?;
        let written = writer(&field.shape, &format!("self.{name}"), 0)?;
        if field.required {
            source.push_str(&format!(
                "        out[{}] = {written}\n",
                literal(&field.name)
            ));
        } else {
            source.push_str(&format!("        if self.{name} is not None:\n"));
            source.push_str(&format!(
                "            out[{}] = {written}\n",
                literal(&field.name)
            ));
        }
    }
    source.push_str("        return out\n");

    Ok(source)
}

/// Fields in the one order a class declares them: what the document requires, then what it does
/// not, since a member carrying a default cannot precede one that carries none.
fn ordered_fields(object: &ObjectShape) -> Vec<&Field> {
    let mut ordered: Vec<&Field> = object
        .fields
        .iter()
        .filter(|field| field.required)
        .collect();
    ordered.extend(object.fields.iter().filter(|field| !field.required));
    ordered
}

/// The problems the API reports, one exception each.
fn errors(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let schema = types.schema(&model.errors.schema)?;
    let discriminant = ident(&model.errors.discriminant, Case::Snake, language, limits)?;
    let base = &types.problem_base;
    let catalogue = &types.problem_enum;
    let read_below: BTreeSet<&str> = BTreeSet::from([schema, catalogue.as_str()]);
    let read_below: Vec<&str> = read_below.into_iter().collect();

    let mut source = format!(
        "\"\"\"The failures the API reports, one exception per problem it can name.\"\"\"\n\n\
         from __future__ import annotations\n\n\
         from {RUNTIME_MODULE} import DecodeError, decode_payload, preview\n\
         from .{MODELS_MODULE} import {}\n\n\n\
         class {base}(Exception):\n    \
         \"\"\"A failure the API answered with, whether or not it could be read as a problem.\"\"\"\n\n    \
         def __init__(self, status: int, problem: {schema} | None, detail: str) -> None:\n        \
         super().__init__(detail)\n        \
         self.status = status\n        \
         self.problem = problem\n",
        read_below.join(", ")
    );

    for (value, declared) in &types.problems {
        source.push_str(&format!(
            "\n\nclass {declared}({base}):\n    \"\"\"{}\"\"\"\n",
            docstring(&format!("The API reported `{value}`."))
        ));
    }

    source.push_str(&format!(
        "\n\nPROBLEMS: dict[{catalogue}, type[{base}]] = {{\n"
    ));
    for (value, declared) in &types.problems {
        let member = ident(value, Case::ScreamingSnake, language, limits)?;
        source.push_str(&format!("    {catalogue}.{member}: {declared},\n"));
    }
    source.push_str("}\n");

    source.push_str(&format!(
        "\n\ndef raise_for_status(status: int, payload: bytes) -> None:\n    \
         \"\"\"Raise what the API reported, when what it answered was not a success.\"\"\"\n    \
         if {LOWEST_SUCCESS} <= status < {LOWEST_REDIRECTION}:\n        \
         return\n\n    \
         try:\n        \
         problem = {schema}.from_json(decode_payload(payload))\n    \
         except DecodeError as unreadable:\n        \
         raise {base}(status, None, f\"the API answered {{status}}: {{preview(payload)}}\") from unreadable\n\n    \
         raise PROBLEMS[problem.{discriminant}](status, problem, f\"the API answered {{status}}: {{problem}}\")\n"
    ));

    Ok(source)
}

/// One method per operation, grouped by the entity its operation id names.
fn requests(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
    flavour: Flavour,
) -> Result<String, Error> {
    let mut needs = Needs::default();
    let mut body = String::new();

    for entity in model.entities.entities() {
        body.push_str(&group(
            entity, types, &mut needs, language, limits, flavour,
        )?);
    }

    let subject = match flavour {
        Flavour::Sync => "waits for its transport",
        Flavour::Async => "awaits its transport",
    };
    let mut header = format!(
        "\"\"\"One method per operation the API declares, grouped by entity. This layer {subject}.\"\"\"\n\n\
         from __future__ import annotations\n\n\
         from typing import Any\n\n"
    );
    header.push_str(&runtime_import(&needs));
    header.push_str(&format!("from .{ERRORS_MODULE} import raise_for_status\n"));
    if !needs.below.is_empty() {
        header.push_str(&format!("from .{MODELS_MODULE} import (\n"));
        for name in &needs.below {
            header.push_str(&format!("    {name},\n"));
        }
        header.push_str(")\n");
    }

    Ok(format!("{header}{body}"))
}

fn group(
    entity: &Entity,
    types: &Types,
    needs: &mut Needs,
    language: &LanguageSpec,
    limits: &Limits,
    flavour: Flavour,
) -> Result<String, Error> {
    let declared = types.group(&entity.name, flavour)?;

    let mut source = format!("\n\nclass {declared}:\n");
    source.push_str(&format!(
        "    \"\"\"{}\"\"\"\n\n",
        docstring(&format!(
            "What the API declares under `{}`, issued through the transport it is handed.",
            entity.name
        ))
    ));
    source.push_str(
        "    def __init__(self, transport: Any) -> None:\n        self._transport = transport\n",
    );

    for method in &entity.methods {
        source.push_str(&operation(method, types, needs, language, limits, flavour)?);
    }

    Ok(source)
}

fn operation(
    method: &Method,
    types: &Types,
    needs: &mut Needs,
    language: &LanguageSpec,
    limits: &Limits,
    flavour: Flavour,
) -> Result<String, Error> {
    let name = ident(&method.verb_text, Case::Snake, language, limits)?;
    let operation = &method.operation;

    let mut path_parameters = Vec::new();
    let mut query_parameters = Vec::new();
    for parameter in &operation.parameters {
        match parameter.location {
            ParameterLocation::Path => path_parameters.push(parameter),
            ParameterLocation::Query => query_parameters.push(parameter),
            ParameterLocation::Header | ParameterLocation::Cookie => {
                return Err(Error::UnsupportedParameter {
                    operation_id: preview(&method.operation_id),
                    parameter: preview(&parameter.name),
                });
            }
        }
    }

    // A path parameter is required by construction, and one that carries no value would leave the
    // path template with a hole in it, so it is asked for before anything that may be left out.
    let mut required = Vec::new();
    let mut optional = Vec::new();
    for parameter in path_parameters.iter().chain(query_parameters.iter()) {
        let argument = (
            ident(&parameter.name, Case::Snake, language, limits)?,
            scalar_annotation(parameter)?,
            *parameter,
        );
        if parameter.required || parameter.location == ParameterLocation::Path {
            required.push(argument);
        } else {
            optional.push(argument);
        }
    }

    let returned = match method.success.as_ref() {
        Some((_, Some(shape))) => annotation(shape, types, needs, 0)?,
        _ => "None".to_owned(),
    };

    let mut source = format!("\n    {} {name}(\n        self,\n", flavour.keyword());
    for (argument, annotated, _) in &required {
        source.push_str(&format!("        {argument}: {annotated},\n"));
    }
    if let Some(shape) = method.request.as_ref() {
        source.push_str(&format!(
            "        body: {},\n",
            annotation(shape, types, needs, 0)?
        ));
    }
    for (argument, annotated, _) in &optional {
        source.push_str(&format!("        {argument}: {annotated} | None = None,\n"));
    }
    source.push_str(&format!("    ) -> {returned}:\n"));
    source.push_str(&format!("        \"\"\"{}\"\"\"\n", summary(method)));

    source.push_str(&format!("        path = {}\n", literal(&operation.path)));
    for (argument, _, parameter) in required
        .iter()
        .filter(|(_, _, parameter)| parameter.location == ParameterLocation::Path)
    {
        source.push_str(&format!(
            "        path = path.replace({}, {}({argument}))\n",
            literal(&format!("{{{}}}", parameter.name)),
            needs.helper("path_segment")
        ));
    }

    source.push_str("        query: list[tuple[str, str]] = []\n");
    for (argument, _, parameter) in required
        .iter()
        .filter(|(_, _, parameter)| parameter.location == ParameterLocation::Query)
    {
        source.push_str(&format!(
            "        query.append(({}, {}({argument})))\n",
            literal(&parameter.name),
            needs.helper("query_value")
        ));
    }
    for (argument, _, parameter) in &optional {
        source.push_str(&format!("        if {argument} is not None:\n"));
        source.push_str(&format!(
            "            query.append(({}, {}({argument})))\n",
            literal(&parameter.name),
            needs.helper("query_value")
        ));
    }

    let sent = match method.request.as_ref() {
        None => "None".to_owned(),
        Some(shape) => writer(shape, "body", 0)?,
    };
    source.push_str(&format!(
        "        status, payload = {}\n            {},\n            path,\n            query,\n            {sent},\n        )\n",
        flavour.call(),
        literal(operation.method.as_str())
    ));
    source.push_str("        raise_for_status(status, payload)\n");

    match method.success.as_ref() {
        Some((_, Some(shape))) => {
            let read = reader(shape, types, needs, 0)?;
            source.push_str(&format!(
                "        return {read}({}(payload))\n",
                needs.helper("decode_payload")
            ));
        }
        _ => source.push_str("        return None\n"),
    }

    Ok(source)
}

/// What the package hands whoever imports it.
fn package(types: &Types) -> String {
    let mut source = String::from(
        "\"\"\"Everything the generator writes, gathered under one import.\"\"\"\n\n\
         from __future__ import annotations\n\n",
    );

    for (module, names) in exports(types) {
        source.push_str(&format!("from .{module} import (\n"));
        for name in names {
            source.push_str(&format!("    {name},\n"));
        }
        source.push_str(")\n");
    }

    source.push_str("\n__all__ = [\n");
    for name in &types.exported {
        source.push_str(&format!("    {},\n", literal(name)));
    }
    source.push_str("]\n");

    source
}

fn exports(types: &Types) -> Vec<(&'static str, BTreeSet<String>)> {
    let mut waiting = BTreeSet::new();
    let mut blocking = BTreeSet::new();
    for (sync, asynchronous) in types.groups.values() {
        blocking.insert(sync.clone());
        waiting.insert(asynchronous.clone());
    }

    let mut failures: BTreeSet<String> = types.problems.values().cloned().collect();
    failures.insert(types.problem_base.clone());

    let mut declared: BTreeSet<String> = types.schemas.values().cloned().collect();
    declared.extend(types.enums.values().cloned());

    // Ordered the way an import block is sorted, so nothing has to be reordered after the fact.
    vec![
        (ASYNC_MODULE, waiting),
        (SYNC_MODULE, blocking),
        (ERRORS_MODULE, failures),
        (MODELS_MODULE, declared),
    ]
}

/// The annotation a value of that shape carries.
fn annotation(
    shape: &Shape,
    types: &Types,
    needs: &mut Needs,
    depth: usize,
) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => {
            if let Some(module) = scalar_module(scalar) {
                needs.module(module);
            }
            types.scalars.of(scalar).to_owned()
        }
        Shape::Array(inner) => format!("list[{}]", annotation(inner, types, needs, depth + 1)?),
        Shape::Map(inner) => {
            format!("dict[str, {}]", annotation(inner, types, needs, depth + 1)?)
        }
        Shape::Enum { name, .. } => {
            let declared = types.enumeration(name)?;
            needs.declared(declared);
            declared.to_owned()
        }
        Shape::Named(name) => {
            let declared = types.schema(name)?;
            needs.declared(declared);
            declared.to_owned()
        }
        Shape::Object(object) => {
            let declared = types.schema(&object.name)?;
            needs.declared(declared);
            declared.to_owned()
        }
        Shape::Json => "Any".to_owned(),
    })
}

/// The standard-library module an annotation of that scalar names, when it names one.
fn scalar_module(scalar: &Scalar) -> Option<&'static str> {
    match scalar {
        Scalar::Uuid => Some("uuid"),
        Scalar::DateTime | Scalar::Date => Some("datetime"),
        Scalar::String
        | Scalar::Url
        | Scalar::Integer32
        | Scalar::Integer64
        | Scalar::Number
        | Scalar::Boolean => None,
    }
}

/// The annotation a parameter travelling in a path or a query carries.
///
/// A parameter of a type no annotation covers stops the emission: sending it under the wrong
/// spelling would be a request the API refuses for a reason nothing in the client explains.
fn scalar_annotation(parameter: &Parameter) -> Result<String, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "str",
        "integer" => "int",
        "number" => "float",
        "boolean" => "bool",
        declared => {
            return Err(Error::UnknownSchemaType {
                subject: preview(&parameter.name),
                declared: preview(declared),
            });
        }
    }
    .to_owned())
}

/// What reads a value of that shape out of what the API answered.
fn reader(shape: &Shape, types: &Types, needs: &mut Needs, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(scalar) => needs.helper(scalar_reader(scalar)).to_owned(),
        Shape::Array(inner) => format!(
            "{}({})",
            needs.helper("as_list"),
            reader(inner, types, needs, depth + 1)?
        ),
        Shape::Map(inner) => format!(
            "{}({})",
            needs.helper("as_map"),
            reader(inner, types, needs, depth + 1)?
        ),
        Shape::Enum { name, .. } => {
            let declared = types.enumeration(name)?;
            needs.declared(declared);
            format!("{}({declared})", needs.helper("as_enum"))
        }
        Shape::Named(name) => {
            let declared = types.schema(name)?;
            needs.declared(declared);
            format!("{declared}.from_json")
        }
        Shape::Object(object) => {
            let declared = types.schema(&object.name)?;
            needs.declared(declared);
            format!("{declared}.from_json")
        }
        Shape::Json => needs.helper("as_json").to_owned(),
    })
}

fn scalar_reader(scalar: &Scalar) -> &'static str {
    match scalar {
        Scalar::String | Scalar::Url => "as_text",
        Scalar::Uuid => "as_uuid",
        Scalar::DateTime => "as_datetime",
        Scalar::Date => "as_date",
        Scalar::Integer32 | Scalar::Integer64 => "as_int",
        Scalar::Number => "as_float",
        Scalar::Boolean => "as_bool",
    }
}

/// What writes `subject` back the way the API reads it.
fn writer(shape: &Shape, subject: &str, depth: usize) -> Result<String, Error> {
    deep_enough(depth)?;

    Ok(match shape {
        Shape::Scalar(Scalar::Uuid) => format!("str({subject})"),
        Shape::Scalar(Scalar::DateTime | Scalar::Date) => format!("{subject}.isoformat()"),
        Shape::Scalar(_) | Shape::Json => subject.to_owned(),
        Shape::Enum { .. } => format!("{subject}.value"),
        Shape::Named(_) | Shape::Object(_) => format!("{subject}.to_json()"),
        Shape::Array(inner) => {
            let item = format!("item{depth}");
            let written = writer(inner, &item, depth + 1)?;
            if written == item {
                format!("list({subject})")
            } else {
                format!("[{written} for {item} in {subject}]")
            }
        }
        Shape::Map(inner) => {
            let key = format!("key{depth}");
            let value = format!("value{depth}");
            let written = writer(inner, &value, depth + 1)?;
            if written == value {
                format!("dict({subject})")
            } else {
                format!("{{{key}: {written} for {key}, {value} in {subject}.items()}}")
            }
        }
    })
}

fn deep_enough(depth: usize) -> Result<(), Error> {
    if depth > Limits::DEFAULT.max_shape_depth {
        return Err(Error::SchemaTooDeep {
            subject: preview("a value of the model"),
            limit: Limits::DEFAULT.max_shape_depth,
        });
    }
    Ok(())
}

/// The name that text is spelled under, out of the way of the language's own vocabulary.
fn ident(
    text: &str,
    case: Case,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let words = checked_words(text, limits)?;
    Ok(escape(&render(&words, case), language.reserved))
}

/// What a method says about itself, from what the document says about the operation.
fn summary(method: &Method) -> String {
    let described = method
        .operation
        .summary
        .as_ref()
        .or(method.operation.description.as_ref());

    match described {
        Some(text) => docstring(text),
        None => docstring(&format!(
            "`{}` on `{}`.",
            method.operation.method, method.operation.path
        )),
    }
}

/// Snapshot text, as a docstring may carry it.
///
/// The snapshot is untrusted input travelling into source: a run of whitespace becomes one space so
/// nothing leaves the line, the quoting characters that would close the docstring early are spelled
/// otherwise, and what is left is cut at a fixed budget.
fn docstring(text: &str) -> String {
    let collapsed: String = text
        .chars()
        .map(|character| {
            if character.is_whitespace() || character.is_control() {
                ' '
            } else if character == '"' || character == '\\' {
                '\''
            } else {
                character
            }
        })
        .collect();

    let mut rendered = String::new();
    let mut spaced = false;
    for character in collapsed.chars() {
        if rendered.chars().count() >= MAX_DOCSTRING_CHARS {
            break;
        }
        if character == ' ' {
            if !rendered.is_empty() {
                spaced = true;
            }
            continue;
        }
        if spaced {
            rendered.push(' ');
            spaced = false;
        }
        rendered.push(character);
    }

    if rendered.is_empty() {
        return "Undocumented by the API.".to_owned();
    }
    rendered
}

/// Snapshot text, as a string literal may carry it.
fn literal(text: &str) -> String {
    let mut rendered = String::from("\"");
    for character in text.chars() {
        match character {
            '\\' => rendered.push_str("\\\\"),
            '"' => rendered.push_str("\\\""),
            '\n' => rendered.push_str("\\n"),
            '\r' => rendered.push_str("\\r"),
            '\t' => rendered.push_str("\\t"),
            control if control.is_control() => {
                rendered.push_str(&format!("\\x{:02x}", control as u32));
            }
            plain => rendered.push(plain),
        }
    }
    rendered.push('"');
    rendered
}

//! Emits the generated half of the Go SDK.
//!
//! The module is fetched by URL with no copy of the OpenAPI snapshot beside it, so the types, the
//! problems and the request layer travel as committed source rather than as a build artefact.
//! Everything the API declares — one type per named schema, one closed list of constants per
//! enumeration, one error value per problem the error contract can report, one method per
//! operation — is written here; everything the API does not declare — how a request reaches the
//! network, how a send is retried, how a webhook signature is verified — is hand-written in the
//! package above this one and never regenerated.
//!
//! The two halves meet at one seam and nowhere else, and that seam is an interface declared *here*.
//! Go satisfies an interface by shape rather than by declaration, so the hand-written transport
//! answers to it without either half importing the other — which is what keeps this package inside
//! the standard library. Nothing here knows what a socket is, and nothing beside it knows what the
//! API declares.
//!
//! What is written is already formatted. `gofmt` lays out consecutive fields, constants and map
//! entries in columns, and the width of a column depends on every line sharing it; a comment breaks
//! a run of such lines, so every field and every constant carries one and no run is ever longer
//! than a single line. Nothing then has to be padded, and the bytes emitted are the bytes `gofmt`
//! would produce.
//!
//! Anything the emitter cannot make sense of — a parameter travelling somewhere a client cannot put
//! it, a type name two declarations would answer to, a scalar no type covers — stops the emission
//! rather than yielding a smaller SDK.

use std::collections::{BTreeMap, BTreeSet};

use crate::emit::{EmittedFile, FileTree, Ownership, RelativePath, banner};
use crate::error::{Error, preview};
use crate::identifier::{Case, spell};
use crate::limits::Limits;
use crate::model::{ApiModel, Entity, ErrorModel, Field, Method, ObjectShape, Scalar, Shape};
use crate::snapshot::{PUBLIC_TAG, Parameter, ParameterLocation};
use crate::targets::{Contract, LanguageSpec, ScalarNames, Target, update_command};

/// How the target is named, and what `UPDATE_SDK` accepts to rewrite it.
pub const NAME: &str = "go";

/// Where the generated half of the module lands.
///
/// Everything under it belongs to the generator; everything beside it is hand-written. The last
/// segment is also what the package is called, since a Go package is reached by the directory it
/// sits in.
const ROOT: &str = "clients/go/generated";

/// The files this target writes, each one holding one layer of the surface.
const DOC_FILE: &str = "doc";
const SCALARS_FILE: &str = "scalars";
const MODELS_FILE: &str = "models";
const ERRORS_FILE: &str = "errors";
const API_FILE: &str = "api";

/// Suffix telling an operation group from a type of the same name: the document names an entity and
/// a schema alike often enough that one of them would otherwise shadow the other.
const GROUP_SUFFIX: &str = "API";

/// What a constructor of an operation group is named with.
const CONSTRUCTOR_PREFIX: &str = "New";

/// Suffix the type every failure of the API is a kind of carries, so a problem and a type spelling
/// the same word stay apart.
const FAILURE_SUFFIX: &str = "Error";

/// Prefix every value naming one entry of the problem catalogue carries, which is how Go spells an
/// error a caller compares against.
const SENTINEL_PREFIX: &str = "Err";

/// Longest fragment of a snapshot description a comment carries.
const MAX_COMMENT_CHARS: usize = 200;

/// Lowest status a response is read as a success under.
const LOWEST_SUCCESS: u16 = 200;

/// Lowest status that is no longer a success.
const LOWEST_REDIRECTION: u16 = 300;

/// The names the scaffolding below declares, which no type of the document may answer to as well.
///
/// They are claimed alongside everything the model names, so a schema called `Transport` is
/// reported as the collision it is rather than emitted as a package that does not compile.
const SCAFFOLDING: [&str; 11] = [
    "Date",
    "Transport",
    "UUID",
    "isDate",
    "isUUID",
    "pathSegment",
    "preview",
    "problemFor",
    "problemSentinel",
    "queryValue",
    "unreadable",
];

/// This target, as the registry carries it.
pub(super) fn target() -> Target {
    Target {
        name: NAME,
        tag: PUBLIC_TAG,
        root: ROOT,
        // The whole directory is generated, so a type the document stopped declaring takes its file
        // with it instead of lingering as an orphan nothing imports.
        ownership: Ownership::Directory,
        contract: Contract::Whole,
        language: super::go(),
        emit,
    }
}

/// Everything the generated half of the module is made of.
fn emit(language: &LanguageSpec, model: &ApiModel) -> Result<FileTree, Error> {
    let limits = Limits::DEFAULT;
    let banner = banner(language.comment, &update_command(NAME), &limits)?;
    let package = package_name(language, &limits)?;

    let enums = model.enumerations(&limits)?;
    let types = Types::read(model, &enums, language, &limits)?;

    let files = vec![
        file(
            DOC_FILE,
            &banner,
            &package,
            &doc(&package),
            language,
            &limits,
        )?,
        file(
            SCALARS_FILE,
            &banner,
            &package,
            SCAFFOLDING_SOURCE,
            language,
            &limits,
        )?,
        file(
            MODELS_FILE,
            &banner,
            &package,
            &models(model, &enums, &types, language, &limits)?,
            language,
            &limits,
        )?,
        file(
            ERRORS_FILE,
            &banner,
            &package,
            &errors(model, &types, language, &limits)?,
            language,
            &limits,
        )?,
        file(
            API_FILE,
            &banner,
            &package,
            &requests(model, &types, language, &limits)?,
            language,
            &limits,
        )?,
    ];

    FileTree::build(files, &limits)
}

/// One file: the banner, the package clause the body opens with, and the body.
fn file(
    stem: &str,
    banner: &str,
    package: &str,
    body: &str,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<EmittedFile, Error> {
    let name = format!("{stem}.{}", language.extension);
    let contents = if body.starts_with("//") {
        // A comment sitting against the package clause is the package's own documentation, which
        // exactly one file of a package carries.
        format!("{banner}\n{body}package {package}\n")
    } else {
        format!("{banner}\npackage {package}\n{body}")
    };

    Ok(EmittedFile {
        path: RelativePath::build(&name, limits)?,
        contents,
    })
}

/// What the package is called: the directory it lands in, spelled the way the language wants a
/// module named.
fn package_name(language: &LanguageSpec, limits: &Limits) -> Result<String, Error> {
    let directory = ROOT.rsplit('/').next().unwrap_or(ROOT);
    ident(directory, language.casing.module, language, limits)
}

/// What the package says about itself, which one file of it carries.
fn doc(package: &str) -> String {
    format!(
        "// Package {package} carries everything the API document describes: one type per schema it\n\
         // declares, one closed list of constants per enumeration it names, one error value per\n\
         // problem it can report, and one method per operation, grouped by the entity its operation\n\
         // id names.\n\
         //\n\
         // Everything the document does not describe — reaching the network, retrying a send,\n\
         // verifying a webhook signature — is hand-written in the package above this one and never\n\
         // regenerated. The two meet at the Transport interface declared here, which the\n\
         // hand-written half answers to by shape, so neither package imports the other and this one\n\
         // reaches nothing outside the standard library.\n"
    )
}

/// The types this language has no name of its own for, and the handful of helpers every emitted
/// method calls, which the snapshot has no say over.
const SCAFFOLDING_SOURCE: &str = r#"
import (
	"encoding/json"
	"fmt"
	"net/url"
	"strings"
	"time"
)

// maxPreviewBytes is the longest fragment of a body a message carries. Bodies are written by a
// server this package does not control, so they are cut at a fixed budget rather than echoed whole
// into whatever the caller logs.
const maxPreviewBytes = 256

// dateLayout is how a day without a time of day is written.
const dateLayout = "2006-01-02"

// UUID is an identifier the document declares as a string of a stated format, which the standard
// library has no type for.
//
// It is a defined string type rather than the sixteen bytes it stands for: the wire form is text,
// keeping that text is what lets a document be read and written back unchanged, and a string is
// comparable, allocates nothing and may key a map. Text that is not an identifier is refused while
// it is being read, so a value of this type is never one that is not.
type UUID string

// String answers the text the identifier travels as.
func (u UUID) String() string {
	return string(u)
}

// UnmarshalJSON reads an identifier, refusing text that does not spell one.
//
// The empty text is this type's own zero, which is what a member the document requires and the API
// did not answer reads as; it is accepted, because a type that refused a value its own decoder
// produces could not write back what it read.
func (u *UUID) UnmarshalJSON(data []byte) error {
	var text string
	if err := json.Unmarshal(data, &text); err != nil {
		return err
	}
	if text != "" && !isUUID(text) {
		return fmt.Errorf("`%s` is not an identifier", preview([]byte(text)))
	}
	*u = UUID(text)
	return nil
}

// isUUID reports whether the text is the canonical hexadecimal form, hyphens included.
func isUUID(text string) bool {
	if len(text) != 36 {
		return false
	}
	for index := 0; index < len(text); index++ {
		character := text[index]
		if index == 8 || index == 13 || index == 18 || index == 23 {
			if character != '-' {
				return false
			}
			continue
		}
		hexadecimal := (character >= '0' && character <= '9') ||
			(character >= 'a' && character <= 'f') ||
			(character >= 'A' && character <= 'F')
		if !hexadecimal {
			return false
		}
	}
	return true
}

// Date is a day the document states without a time of day, which the standard library has no type
// for either: time.Time carries a moment, and writing one back would add to the document a time the
// API never sent.
type Date string

// String answers the text the day travels as.
func (d Date) String() string {
	return string(d)
}

// UnmarshalJSON reads a day, refusing text that does not spell one.
//
// The empty text is this type's own zero, and is accepted for the same reason UUID accepts it.
func (d *Date) UnmarshalJSON(data []byte) error {
	var text string
	if err := json.Unmarshal(data, &text); err != nil {
		return err
	}
	if text != "" && !isDate(text) {
		return fmt.Errorf("`%s` is not a day", preview([]byte(text)))
	}
	*d = Date(text)
	return nil
}

// isDate reports whether the text is a day, month and year that exist.
func isDate(text string) bool {
	_, err := time.Parse(dateLayout, text)
	return err == nil
}

// preview is as much of a body as a message may carry, with whatever was not text spelled as the
// replacement character rather than carried into a log as bytes.
func preview(payload []byte) string {
	if len(payload) <= maxPreviewBytes {
		return strings.ToValidUTF8(string(payload), "�")
	}
	return strings.ToValidUTF8(string(payload[:maxPreviewBytes]), "�") + "…"
}

// unreadable says that the API answered something the document does not describe.
func unreadable(status int, payload []byte, cause error) error {
	return fmt.Errorf("the API answered %d with a body this client cannot read (%s): %w", status, preview(payload), cause)
}

// queryValue writes a value the way a request line carries it, which is not always how Go prints
// it.
func queryValue(value any) string {
	if text, ok := value.(string); ok {
		return text
	}
	return fmt.Sprint(value)
}

// pathSegment writes a value as one segment of a path, with nothing left in it that could name
// another one.
func pathSegment(value any) string {
	return url.PathEscape(queryValue(value))
}
"#;

/// Every name the generated package declares, under the layer that declares it.
///
/// Names are settled before a single line is written, so a collision is reported as a collision
/// rather than as a package that will not compile.
struct Types {
    /// Type each named schema is declared under, by the name the model gives it.
    schemas: BTreeMap<String, String>,
    /// Type each closed list of strings is declared under.
    enums: BTreeMap<String, String>,
    /// Value each problem is reported as, by the value the catalogue lists.
    problems: BTreeMap<String, String>,
    /// Type every failure of the API is a kind of.
    failure: String,
    /// Type the discriminant of the error contract carries.
    problem_enum: String,
    /// Operation groups, by entity name.
    groups: BTreeMap<String, String>,
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

        for declared in SCAFFOLDING {
            claim(declared.to_owned(), "the scaffolding this target writes")?;
        }

        let mut schemas = BTreeMap::new();
        for name in model.schemas.keys() {
            let declared = claim(
                ident(name, language.casing.type_name, language, limits)?,
                name,
            )?;
            schemas.insert(name.clone(), declared);
        }

        let mut declared_enums = BTreeMap::new();
        for name in enums.keys() {
            let declared = claim(
                ident(name, language.casing.type_name, language, limits)?,
                name,
            )?;
            declared_enums.insert(name.clone(), declared);
        }

        let problem_enum = enum_of(&model.errors, &declared_enums)?;

        let base = format!(
            "{}{FAILURE_SUFFIX}",
            ident(
                &model.errors.schema,
                language.casing.type_name,
                language,
                limits
            )?
        );
        let failure = claim(base, &model.errors.schema)?;

        let mut problems = BTreeMap::new();
        for value in model.errors.catalogue.values() {
            let name = format!(
                "{SENTINEL_PREFIX}{}",
                ident(value, language.casing.type_name, language, limits)?
            );
            problems.insert(value.clone(), claim(name, value)?);
        }

        let mut groups = BTreeMap::new();
        for entity in model.entities.entities() {
            let stem = ident(&entity.name, language.casing.type_name, language, limits)?;
            let declared = claim(format!("{stem}{GROUP_SUFFIX}"), &entity.name)?;
            claim(format!("{CONSTRUCTOR_PREFIX}{declared}"), &entity.name)?;
            groups.insert(entity.name.clone(), declared);
        }

        Ok(Self {
            schemas,
            enums: declared_enums,
            problems,
            failure,
            problem_enum,
            groups,
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

    fn group(&self, entity: &str) -> Result<&str, Error> {
        self.groups
            .get(entity)
            .map(String::as_str)
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
    let discriminant = discriminant_field(errors)?;

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

/// The member of the error schema that tells one problem from another.
fn discriminant_field(errors: &ErrorModel) -> Result<&Field, Error> {
    errors
        .shape
        .fields
        .iter()
        .find(|field| field.name == errors.discriminant)
        .ok_or_else(|| Error::ErrorSchemaWithoutCatalogue {
            schema: preview(&errors.schema),
        })
}

/// What a file imports, gathered while its body is written rather than guessed afterwards.
///
/// An import a Go file does not use is not a warning, it is a file that will not compile, so this
/// is the difference between a package and a broken one.
#[derive(Debug, Default)]
struct Needs {
    modules: BTreeSet<&'static str>,
}

impl Needs {
    fn module(&mut self, name: &'static str) {
        self.modules.insert(name);
    }

    /// The import block, or nothing at all when the file reaches for nothing.
    fn block(&self) -> String {
        if self.modules.is_empty() {
            return String::new();
        }

        let mut block = String::from("\nimport (\n");
        for module in &self.modules {
            block.push_str(&format!("\t\"{module}\"\n"));
        }
        block.push_str(")\n");
        block
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

    Ok(format!("{}{body}", needs.block()))
}

/// One closed list of strings, as a type of its own and one constant per value it admits.
fn enumeration(
    declared: &str,
    values: &[String],
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut members: BTreeMap<String, &String> = BTreeMap::new();
    let mut source = format!(
        "\n// {declared} is one of the values the API answers with.\ntype {declared} string\n\nconst (\n"
    );

    for value in values {
        let member = format!(
            "{declared}{}",
            ident(value, language.casing.constant, language, limits)?
        );
        if let Some(first) = members.insert(member.clone(), value) {
            return Err(Error::SchemaNameCollision {
                name: preview(&member),
                first: preview(first),
                second: preview(value),
            });
        }
        source.push_str(&format!(
            "\t// {member} is the `{}` the API answers with.\n\t{member} {declared} = {}\n",
            comment(value),
            literal(value)
        ));
    }
    source.push_str(")\n");

    Ok(source)
}

/// One named schema, as the type a caller reads and writes.
fn structure(
    declared: &str,
    object: &ObjectShape,
    types: &Types,
    needs: &mut Needs,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut source = format!(
        "\n// {declared} is the `{}` the API declares.\ntype {declared} struct {{\n",
        comment(&object.name)
    );

    let mut claimed: BTreeMap<String, &str> = BTreeMap::new();
    for field in &object.fields {
        let name = ident(&field.name, language.casing.field, language, limits)?;
        if let Some(first) = claimed.insert(name.clone(), &field.name) {
            return Err(Error::SchemaNameCollision {
                name: preview(&name),
                first: preview(first),
                second: preview(&field.name),
            });
        }

        let declared_type = annotation(&field.shape, field.required, types, needs, 0)?;
        source.push_str(&format!(
            "\t// {name} carries `{}`{}\n\t{name} {declared_type} `json:\"{}{}\"`\n",
            comment(&field.name),
            described(field.description.as_deref()),
            tag(&field.name)?,
            if field.required { "" } else { ",omitempty" }
        ));
    }
    source.push_str("}\n");

    Ok(source)
}

/// What a field says about itself beyond the name it carries, when the document says anything.
fn described(description: Option<&str>) -> String {
    match description {
        Some(text) => format!(": {}", comment(text)),
        None => ".".to_owned(),
    }
}

/// A wire name as a struct tag may carry it.
///
/// The tag is read back by the encoder as a comma-separated list inside a quoted string, so a name
/// carrying either of those, or anything a Go string literal would have to escape, is refused: a
/// tag the encoder reads differently from what the document says is a field that silently travels
/// under the wrong name.
fn tag(name: &str) -> Result<String, Error> {
    let unusable = name.is_empty()
        || name.chars().any(|character| {
            character == '"'
                || character == ','
                || character == '\\'
                || character == '`'
                || character.is_control()
                || character.is_whitespace()
        });

    if unusable {
        return Err(Error::UnsafePath {
            path: preview(name),
            reason: "it cannot be written as the name a field travels under".to_owned(),
        });
    }

    Ok(name.to_owned())
}

/// The problems the API reports, one value each.
fn errors(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let schema = types.schema(&model.errors.schema)?;
    let discriminant = discriminant_field(&model.errors)?;
    let read = ident(
        &model.errors.discriminant,
        language.casing.field,
        language,
        limits,
    )?;
    let failure = &types.failure;
    let catalogue = &types.problem_enum;

    let mut needs = Needs::default();
    needs.module("encoding/json");
    needs.module("fmt");

    let mut source = format!(
        "{}\n\
         // {failure} is what the API answers with when it does not answer a success.\n\
         //\n\
         // Every problem the document names is one of these, told apart by Kind and compared\n\
         // against the value declared for it with errors.Is; the document the API sent, when it\n\
         // sent one this client can read, is under Problem. A body naming no problem still reaches\n\
         // a caller as one of these, carrying no Kind.\n\
         type {failure} struct {{\n\
         \t// Status is what the API answered under.\n\
         \tStatus int\n\
         \t// Kind is the problem the API named, empty when the body named none.\n\
         \tKind {catalogue}\n\
         \t// Problem is the document the API answered, nil when it answered none this client can read.\n\
         \tProblem *{schema}\n\
         \t// Detail is what to say about the failure, as much of the API's answer as fits included.\n\
         \tDetail string\n\
         }}\n\n\
         // Error says what the API answered.\n\
         func (e *{failure}) Error() string {{\n\
         \treturn e.Detail\n\
         }}\n\n\
         // Is reports whether this failure is the problem the target names, which is what lets\n\
         // errors.Is compare it against the value declared for each entry of the catalogue.\n\
         func (e *{failure}) Is(target error) bool {{\n\
         \tnamed, ok := target.(problemSentinel)\n\
         \treturn ok && {catalogue}(named) == e.Kind\n\
         }}\n\n\
         // problemSentinel names one entry of the catalogue as a value errors.Is compares against.\n\
         // It is a string rather than a pointer, so the values declared below cannot be written\n\
         // through by whoever holds one.\n\
         type problemSentinel {catalogue}\n\n\
         // Error names the problem, so that a value read on its own still says something.\n\
         func (p problemSentinel) Error() string {{\n\
         \treturn string(p)\n\
         }}\n",
        needs.block()
    );

    for (value, declared) in &types.problems {
        let member = format!(
            "{catalogue}{}",
            ident(value, language.casing.constant, language, limits)?
        );
        source.push_str(&format!(
            "\n// {declared} is the `{}` the API reports.\nvar {declared} error = problemSentinel({member})\n",
            comment(value)
        ));
    }

    // The discriminant is what says whether a body named a problem at all, and a document may
    // declare it as a member it does not require.
    let (named, taken) = if discriminant.required {
        (format!("problem.{read} == \"\""), format!("problem.{read}"))
    } else {
        (format!("problem.{read} == nil"), format!("*problem.{read}"))
    };

    source.push_str(&format!(
        "\n// problemFor answers the failure the API reported, and nothing at all when what it\n\
         // answered was a success.\n\
         func problemFor(status int, payload []byte) error {{\n\
         \tif status >= {LOWEST_SUCCESS} && status < {LOWEST_REDIRECTION} {{\n\
         \t\treturn nil\n\
         \t}}\n\n\
         \tfailure := &{failure}{{\n\
         \t\tStatus: status,\n\
         \t\tDetail: fmt.Sprintf(\"the API answered %d: %s\", status, preview(payload)),\n\
         \t}}\n\n\
         \tvar problem {schema}\n\
         \tif err := json.Unmarshal(payload, &problem); err != nil || {named} {{\n\
         \t\treturn failure\n\
         \t}}\n\n\
         \tfailure.Kind = {taken}\n\
         \tfailure.Problem = &problem\n\
         \treturn failure\n\
         }}\n"
    ));

    Ok(source)
}

/// One method per operation, grouped by the entity its operation id names.
fn requests(
    model: &ApiModel,
    types: &Types,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let mut needs = Needs::default();
    needs.module("context");
    needs.module("net/url");

    let mut body = String::new();
    for entity in model.entities.entities() {
        body.push_str(&group(entity, types, &mut needs, language, limits)?);
    }

    let header = format!(
        "{}\n\
         // Transport is what a generated method issues its request through.\n\
         //\n\
         // It is declared here rather than imported: Go answers to an interface by shape, so the\n\
         // hand-written transport beside this package satisfies it without either half naming the\n\
         // other, and this package therefore reaches nothing outside the standard library.\n\
         type Transport interface {{\n\
         \t// Request issues one request, and answers the status, the body, and why it got neither.\n\
         \tRequest(ctx context.Context, method string, path string, query url.Values, body any) (int, []byte, error)\n\
         }}\n",
        needs.block()
    );

    Ok(format!("{header}{body}"))
}

fn group(
    entity: &Entity,
    types: &Types,
    needs: &mut Needs,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let declared = types.group(&entity.name)?;

    let mut source = format!(
        "\n// {declared} is what the API declares under `{}`.\n\
         //\n\
         // Every method of it is issued through the transport it is handed.\n\
         type {declared} struct {{\n\
         \ttransport Transport\n\
         }}\n\n\
         // {CONSTRUCTOR_PREFIX}{declared} reaches what the API declares under `{}`.\n\
         func {CONSTRUCTOR_PREFIX}{declared}(transport Transport) *{declared} {{\n\
         \treturn &{declared}{{transport: transport}}\n\
         }}\n",
        comment(&entity.name),
        comment(&entity.name)
    );

    for method in &entity.methods {
        source.push_str(&operation(
            method, declared, types, needs, language, limits,
        )?);
    }

    Ok(source)
}

/// What a method answers besides the failure it may report.
enum Returns {
    /// Nothing but whether it worked.
    Nothing,
    /// A value handed back by reference, which is how Go carries a type a caller may hold on to.
    Reference(String),
    /// A value handed back as it stands, which is what a list, a map or a scalar is.
    Value(String),
}

impl Returns {
    /// How the return list of the method reads.
    fn list(&self) -> String {
        match self {
            Self::Nothing => "error".to_owned(),
            Self::Reference(declared) => format!("(*{declared}, error)"),
            Self::Value(declared) => format!("({declared}, error)"),
        }
    }

    /// What is answered beside a failure, which is nothing a caller should read.
    fn nothing(&self) -> &'static str {
        match self {
            Self::Nothing => "",
            Self::Reference(_) => "nil, ",
            Self::Value(_) => "out, ",
        }
    }
}

fn operation(
    method: &Method,
    group: &str,
    types: &Types,
    needs: &mut Needs,
    language: &LanguageSpec,
    limits: &Limits,
) -> Result<String, Error> {
    let name = ident(&method.verb_text, language.casing.method, language, limits)?;
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
            ident(&parameter.name, language.casing.parameter, language, limits)?,
            scalar_annotation(parameter)?,
            *parameter,
        );
        if parameter.required || parameter.location == ParameterLocation::Path {
            required.push(argument);
        } else {
            optional.push(argument);
        }
    }

    let returns = match method.success.as_ref() {
        Some((_, Some(shape))) => returned(shape, types, needs)?,
        _ => Returns::Nothing,
    };

    let mut source = format!(
        "\n// {name} is what the API declares as `{}`, `{} {}`.{}\n\
         func ({group_receiver} *{group}) {name}(\n\
         \tctx context.Context,\n",
        comment(&method.operation_id),
        operation.method,
        comment(&operation.path),
        summary(method),
        group_receiver = RECEIVER,
    );
    for (argument, annotated, _) in &required {
        source.push_str(&format!("\t{argument} {annotated},\n"));
    }
    if let Some(shape) = method.request.as_ref() {
        source.push_str(&format!(
            "\tbody {},\n",
            annotation(shape, true, types, needs, 0)?
        ));
    }
    for (argument, annotated, _) in &optional {
        source.push_str(&format!("\t{argument} *{annotated},\n"));
    }
    source.push_str(&format!(") {} {{\n", returns.list()));

    source.push_str(&format!("\tpath := {}\n", literal(&operation.path)));
    for (argument, _, parameter) in required
        .iter()
        .filter(|(_, _, parameter)| parameter.location == ParameterLocation::Path)
    {
        needs.module("strings");
        source.push_str(&format!(
            "\tpath = strings.ReplaceAll(path, {}, pathSegment({argument}))\n",
            literal(&format!("{{{}}}", parameter.name))
        ));
    }

    source.push_str("\tquery := url.Values{}\n");
    for (argument, _, parameter) in required
        .iter()
        .filter(|(_, _, parameter)| parameter.location == ParameterLocation::Query)
    {
        source.push_str(&format!(
            "\tquery.Set({}, queryValue({argument}))\n",
            literal(&parameter.name)
        ));
    }
    for (argument, _, parameter) in &optional {
        source.push_str(&format!(
            "\tif {argument} != nil {{\n\t\tquery.Set({}, queryValue(*{argument}))\n\t}}\n",
            literal(&parameter.name)
        ));
    }

    if let Returns::Reference(declared) | Returns::Value(declared) = &returns {
        source.push_str(&format!("\n\tvar out {declared}\n"));
    }

    let sent = if method.request.is_some() {
        "body"
    } else {
        "nil"
    };
    source.push_str(&format!(
        "\tstatus, payload, err := {RECEIVER}.transport.Request(ctx, {}, path, query, {sent})\n\
         \tif err != nil {{\n\
         \t\treturn {}err\n\
         \t}}\n",
        literal(operation.method.as_str()),
        returns.nothing()
    ));

    match &returns {
        Returns::Nothing => source.push_str("\treturn problemFor(status, payload)\n"),
        _ => {
            needs.module("encoding/json");
            source.push_str(&format!(
                "\tif failure := problemFor(status, payload); failure != nil {{\n\
                 \t\treturn {nothing}failure\n\
                 \t}}\n\n\
                 \tif err := json.Unmarshal(payload, &out); err != nil {{\n\
                 \t\treturn {nothing}unreadable(status, payload, err)\n\
                 \t}}\n\
                 \treturn {answered}out, nil\n",
                nothing = returns.nothing(),
                answered = match returns {
                    Returns::Reference(_) => "&",
                    _ => "",
                }
            ));
        }
    }
    source.push_str("}\n");

    Ok(source)
}

/// What every emitted method calls its own receiver.
///
/// It is one of the words the language spec reserves, so an argument the document names the same
/// way is spelled out of its way rather than assigned over.
const RECEIVER: &str = "group";

/// What a method hands back when it worked.
fn returned(shape: &Shape, types: &Types, needs: &mut Needs) -> Result<Returns, Error> {
    let declared = annotation(shape, true, types, needs, 0)?;

    Ok(match shape {
        // Only a type a caller may hold on to is worth a reference; a list, a map or a scalar is
        // already the value it stands for, and handing one back by reference would be a nil to
        // check for nothing.
        Shape::Named(_) | Shape::Object(_) => Returns::Reference(declared),
        _ => Returns::Value(declared),
    })
}

/// The type a value of that shape carries.
///
/// Optionality is membership in `required` and nothing else. A shape whose Go type already has an
/// absent value of its own — a list, a map, a value the document does not describe — says so with
/// that value; everything else says so by reference, which is the only way Go tells a member that
/// was not answered from one answered as its zero.
fn annotation(
    shape: &Shape,
    required: bool,
    types: &Types,
    needs: &mut Needs,
    depth: usize,
) -> Result<String, Error> {
    deep_enough(depth)?;

    let declared = match shape {
        Shape::Scalar(scalar) => {
            if let Some(module) = scalar_module(scalar) {
                needs.module(module);
            }
            types.scalars.of(scalar).to_owned()
        }
        Shape::Array(inner) => format!("[]{}", annotation(inner, true, types, needs, depth + 1)?),
        Shape::Map(inner) => format!(
            "map[string]{}",
            annotation(inner, true, types, needs, depth + 1)?
        ),
        Shape::Enum { name, .. } => types.enumeration(name)?.to_owned(),
        Shape::Named(name) => types.schema(name)?.to_owned(),
        Shape::Object(object) => types.schema(&object.name)?.to_owned(),
        Shape::Json => "any".to_owned(),
    };

    if required || carries_its_own_absence(shape) {
        return Ok(declared);
    }
    Ok(format!("*{declared}"))
}

/// Whether the Go type of that shape already has a value standing for a member that was not
/// answered.
fn carries_its_own_absence(shape: &Shape) -> bool {
    matches!(shape, Shape::Array(_) | Shape::Map(_) | Shape::Json)
}

/// The standard-library package a type names, when it names one.
fn scalar_module(scalar: &Scalar) -> Option<&'static str> {
    match scalar {
        Scalar::DateTime => Some("time"),
        Scalar::String
        | Scalar::Uuid
        | Scalar::Date
        | Scalar::Url
        | Scalar::Integer32
        | Scalar::Integer64
        | Scalar::Number
        | Scalar::Boolean => None,
    }
}

/// The type a parameter travelling in a path or a query carries.
///
/// A parameter of a type nothing covers stops the emission: sending it under the wrong spelling
/// would be a request the API refuses for a reason nothing in the client explains.
fn scalar_annotation(parameter: &Parameter) -> Result<String, Error> {
    Ok(match parameter.schema_type.as_str() {
        "string" => "string",
        "integer" => "int64",
        "number" => "float64",
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
    spell(text, case, language.reserved, limits)
}

/// What a method says about itself, from what the document says about the operation.
fn summary(method: &Method) -> String {
    let described = method
        .operation
        .summary
        .as_ref()
        .or(method.operation.description.as_ref());

    match described {
        Some(text) => format!("\n//\n// {}", comment(text)),
        None => String::new(),
    }
}

/// Snapshot text, as a comment may carry it.
///
/// The snapshot is untrusted input travelling into source: a run of whitespace becomes one space so
/// nothing leaves the line the comment sits on, and what is left is cut at a fixed budget.
fn comment(text: &str) -> String {
    let collapsed: String = text
        .chars()
        .map(|character| {
            if character.is_whitespace() || character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect();

    let mut rendered = String::new();
    let mut spaced = false;
    for character in collapsed.chars() {
        if rendered.chars().count() >= MAX_COMMENT_CHARS {
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
        return "undocumented by the API".to_owned();
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
            // Spelled as the code point rather than as the byte it would be written as, so that a
            // literal never carries a byte that is not text.
            control if control.is_control() => {
                rendered.push_str(&format!("\\u{:04x}", control as u32));
            }
            plain => rendered.push(plain),
        }
    }
    rendered.push('"');
    rendered
}

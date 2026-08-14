//! What the generator writes, stated as data rather than as code.
//!
//! A target is a name, the tag it selects out of the snapshot, where it lands, what it owns there,
//! how its language spells things, and one function that turns the model into files. Nothing about
//! a target is discovered: [`targets`] *is* the list of what can be generated, which is what lets
//! one driver iterate it — and what makes the values [`UPDATE_VARIABLE`] accepts the registry
//! itself rather than a list written beside it.
//!
//! A language is described once, in a [`LanguageSpec`], and handed to every emitter of that
//! language: two emitters writing the same language cannot disagree on how a field is spelled or on
//! what a `date-time` is called.

pub mod go;
pub mod mcp;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod typescript;

use std::sync::LazyLock;

use crate::emit::{CommentStyle, FileTree, Ownership};
use crate::error::Error;
use crate::identifier::{Case, Casing, Escape, ReservedWords};
use crate::model::{ApiModel, Scalar};

/// The variable that turns the emission driver from a drift guard into a rewrite.
pub const UPDATE_VARIABLE: &str = "UPDATE_SDK";

/// The test the variable is set on.
pub const UPDATE_TEST: &str = "cargo test -p hook0-sdkgen sdk_targets";

/// The value naming every target at once, which is therefore not a name a target may answer to.
pub const EVERY_TARGET: &str = "1";

/// What to run to adopt a deliberate change of one target.
pub fn update_command(target: &str) -> String {
    format!("{UPDATE_VARIABLE}={target} {UPDATE_TEST}")
}

/// The type name each scalar of the model carries in one language.
///
/// A scalar the model knows about and a language has no name for cannot happen: the table is
/// exhaustive, so widening [`Scalar`] stops every target that has not been told what to call the
/// new one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScalarNames {
    pub string: &'static str,
    pub uuid: &'static str,
    pub date_time: &'static str,
    pub date: &'static str,
    pub url: &'static str,
    pub integer32: &'static str,
    pub integer64: &'static str,
    pub number: &'static str,
    pub boolean: &'static str,
}

impl ScalarNames {
    /// What this language calls that scalar.
    pub fn of(&self, scalar: &Scalar) -> &'static str {
        match scalar {
            Scalar::String => self.string,
            Scalar::Uuid => self.uuid,
            Scalar::DateTime => self.date_time,
            Scalar::Date => self.date,
            Scalar::Url => self.url,
            Scalar::Integer32 => self.integer32,
            Scalar::Integer64 => self.integer64,
            Scalar::Number => self.number,
            Scalar::Boolean => self.boolean,
        }
    }
}

/// Everything one language wants of the names and types it is handed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguageSpec {
    pub casing: Casing,
    pub reserved: &'static ReservedWords,
    pub scalars: ScalarNames,
    pub comment: CommentStyle,
    /// What a file of this language is named with, without the dot.
    pub extension: &'static str,
}

/// One thing the generator writes.
#[derive(Debug)]
pub struct Target {
    /// How the target is named, which is also what [`UPDATE_VARIABLE`] accepts for it.
    pub name: &'static str,
    /// The tag this target selects out of the snapshot.
    pub tag: &'static str,
    /// Where the target lands, relative to the root of the repository.
    pub root: &'static str,
    pub ownership: Ownership,
    pub language: LanguageSpec,
    /// Turns the model into the files the target is made of.
    ///
    /// A function rather than a trait object: a target carries no state, so there is nothing for an
    /// implementation to hold, and the registry stays a plain list of values.
    pub emit: fn(&LanguageSpec, &ApiModel) -> Result<FileTree, Error>,
}

/// Everything the generator writes, in the order the driver walks it.
pub fn targets() -> &'static [Target] {
    &TARGETS
}

static TARGETS: LazyLock<Vec<Target>> = LazyLock::new(|| {
    vec![
        go::target(),
        mcp::target(),
        python::target(),
        ruby::target(),
        rust::target(),
        typescript::target(),
    ]
});

/// Rust's own vocabulary, including the words it reserves without using them yet.
///
/// Sorted, as [`ReservedWords`] searches it by halving.
const RUST_KEYWORDS: [&str; 49] = [
    "as", "async", "await", "become", "box", "break", "const", "continue", "crate", "do", "dyn",
    "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in", "let", "loop",
    "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return", "self",
    "static", "struct", "super", "trait", "true", "try", "type", "typeof", "unsafe", "unsized",
    "use", "virtual", "where", "while", "yield",
];

/// What a name rendering to no identifier at all is spelled as in Rust.
const RUST_PLACEHOLDER: &str = "value";

static RUST_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&RUST_KEYWORDS, Escape::Suffix('_'), RUST_PLACEHOLDER).expect(
        "the Rust keyword table is sorted, carries no empty word, and does not hold its own fallback",
    )
});

/// How Rust wants what it is handed spelled.
fn rust() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::Snake,
            field: Case::Snake,
            parameter: Case::Snake,
            constant: Case::ScreamingSnake,
            file: Case::Snake,
            module: Case::Snake,
        },
        reserved: LazyLock::force(&RUST_RESERVED),
        scalars: ScalarNames {
            string: "String",
            uuid: "Uuid",
            date_time: "DateTime<Utc>",
            date: "NaiveDate",
            url: "Url",
            integer32: "i32",
            integer64: "i64",
            number: "f64",
            boolean: "bool",
        },
        comment: CommentStyle::DoubleSlash,
        extension: "rs",
    }
}

/// Python's own vocabulary, plus the two builtins the API surface actually collides with.
///
/// The keywords are the words no identifier may be spelled as at all; `id` and `type` are not
/// keywords, but the document names fields after both, and a member shadowing the builtin it is
/// named after is a name whose meaning depends on where it is read. Sorted, as [`ReservedWords`]
/// searches it by halving — which puts the capitalised constants first.
const PYTHON_KEYWORDS: [&str; 37] = [
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "id", "if",
    "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "type", "while", "with", "yield",
];

/// What a name rendering to no identifier at all is spelled as in Python.
const PYTHON_PLACEHOLDER: &str = "value";

static PYTHON_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&PYTHON_KEYWORDS, Escape::Suffix('_'), PYTHON_PLACEHOLDER).expect(
        "the Python keyword table is sorted, carries no empty word, and does not hold its own fallback",
    )
});

/// How Python wants what it is handed spelled.
fn python() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::Snake,
            field: Case::Snake,
            parameter: Case::Snake,
            constant: Case::ScreamingSnake,
            file: Case::Snake,
            module: Case::Snake,
        },
        reserved: LazyLock::force(&PYTHON_RESERVED),
        scalars: ScalarNames {
            string: "str",
            uuid: "uuid.UUID",
            date_time: "datetime.datetime",
            date: "datetime.date",
            url: "str",
            integer32: "int",
            integer64: "int",
            number: "float",
            boolean: "bool",
        },
        comment: CommentStyle::Hash,
        extension: "py",
    }
}

/// Ruby's own vocabulary: the words the language reserves for itself.
///
/// Every one of them is a legal *method* name — `def next` and `p.next(1)` both work, because the
/// parser reads what follows `def` and `.` as a name rather than as a keyword. What they cannot be
/// is a member: a member is read back inside the constructor that takes it, and `@until = until`
/// is the keyword, not the argument. That asymmetry is why this table and [`RUBY_SHADOWED`] are
/// kept apart rather than merged into one list applied everywhere.
///
/// Sorted, as [`ReservedWords`] searches it by halving.
const RUBY_KEYWORDS: [&str; 41] = [
    "BEGIN",
    "END",
    "__ENCODING__",
    "__FILE__",
    "__LINE__",
    "alias",
    "and",
    "begin",
    "break",
    "case",
    "class",
    "def",
    "defined?",
    "do",
    "else",
    "elsif",
    "end",
    "ensure",
    "false",
    "for",
    "if",
    "in",
    "module",
    "next",
    "nil",
    "not",
    "or",
    "redo",
    "rescue",
    "retry",
    "return",
    "self",
    "super",
    "then",
    "true",
    "undef",
    "unless",
    "until",
    "when",
    "while",
    "yield",
];

/// The names an emitted declaration would *replace* rather than merely sit beside.
///
/// These are what `Object.new.methods` answers once `json` is loaded — which the emitted code
/// always is, so `to_json` belongs here and a member spelled that way would stop `JSON.generate`
/// working on anything holding the value — plus `initialize`, the `to_h` every emitted value
/// declares, and the two helpers every emitted operation group declares. None of them is reserved
/// by the language; each of them is already answered by every object, so a member or a method
/// spelled that way takes the place of the one that was there. A value whose `hash` is a string it
/// happens to carry cannot key a hash, and a group whose `class` names an operation no longer says
/// what it is.
///
/// What is deliberately absent is Kernel's private vocabulary — `format`, `raise`, `puts`,
/// `require` and the rest. Those are shadowed only inside the class declaring a member of that
/// name, and nothing emitted inside such a class ever calls one, so reserving them would rename a
/// member the API is entitled to declare for a collision that cannot happen. `id` and `type` are
/// absent for a stronger reason still: neither has been a method of `Object` since Ruby 1.9, and
/// the document names fields after both.
///
/// Sorted, as [`ReservedWords`] searches it by halving.
const RUBY_SHADOWED: [&str; 41] = [
    "__id__",
    "__send__",
    "check_answer",
    "class",
    "clone",
    "define_singleton_method",
    "display",
    "dup",
    "enum_for",
    "extend",
    "freeze",
    "hash",
    "initialize",
    "inspect",
    "instance_eval",
    "instance_exec",
    "instance_variable_get",
    "instance_variable_set",
    "instance_variables",
    "itself",
    "method",
    "methods",
    "object_id",
    "private_methods",
    "protected_methods",
    "public_method",
    "public_methods",
    "public_send",
    "read_answer",
    "remove_instance_variable",
    "send",
    "singleton_class",
    "singleton_method",
    "singleton_methods",
    "tap",
    "then",
    "to_enum",
    "to_h",
    "to_json",
    "to_s",
    "yield_self",
];

/// What a name rendering to no identifier at all is spelled as in Ruby.
const RUBY_PLACEHOLDER: &str = "value";

/// Everything a member of a Ruby declaration has to stay out of the way of: the language's own
/// words and the names every object already answers to, merged into the one sorted list
/// [`ReservedWords`] searches. `class` and `then` sit in both halves, so the merge deduplicates.
static RUBY_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    let mut words: Vec<&str> = RUBY_KEYWORDS
        .iter()
        .chain(RUBY_SHADOWED.iter())
        .copied()
        .collect();
    words.sort_unstable();
    words.dedup();

    ReservedWords::build(&words, Escape::Suffix('_'), RUBY_PLACEHOLDER).expect(
        "the merged Ruby vocabulary is sorted, carries no empty word, and does not hold its own fallback",
    )
});

/// What a *method* of an emitted declaration has to stay out of the way of, which is the shadowing
/// half alone: escaping a method named after a keyword would spell `next_` where `next` is a name
/// Ruby accepts and a caller can write, and that wart would ship in a published gem for good.
static RUBY_METHOD_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&RUBY_SHADOWED, Escape::Suffix('_'), RUBY_PLACEHOLDER).expect(
        "the Ruby shadowing table is sorted, carries no empty word, and does not hold its own fallback",
    )
});

/// The vocabulary a method of an emitted Ruby declaration is spelled out of the way of.
pub(super) fn ruby_method_reserved() -> &'static ReservedWords {
    LazyLock::force(&RUBY_METHOD_RESERVED)
}

/// How Ruby wants what it is handed spelled.
///
/// The scalar names are what a documentation comment says a value is, since the source itself says
/// nothing: Ruby has no type for an identifier or for a URL, and both travel as the text the API
/// answers. `Boolean` names no class at all — it is the convention every Ruby documentation tool
/// reads for a value that is `true` or `false`.
fn ruby() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::Snake,
            field: Case::Snake,
            parameter: Case::Snake,
            constant: Case::ScreamingSnake,
            file: Case::Snake,
            module: Case::UpperCamel,
        },
        reserved: LazyLock::force(&RUBY_RESERVED),
        scalars: ScalarNames {
            string: "String",
            uuid: "String",
            date_time: "Time",
            date: "Date",
            url: "String",
            integer32: "Integer",
            integer64: "Integer",
            number: "Float",
            boolean: "Boolean",
        },
        comment: CommentStyle::Hash,
        extension: "rb",
    }
}

/// Go's own vocabulary: its keywords, its predeclared identifiers, and the words the emitter keeps
/// for the locals it writes into every method body.
///
/// The predeclared ones are not keywords — nothing stops an argument called `len` or `error` — but
/// one shadows the builtin for the rest of the body it sits in, and a body that calls `len` after
/// that calls whatever was passed. The emitter's own locals are here for the same reason: an
/// argument spelling `path` would be assigned over by the line that fills in the path template.
/// Sorted, as [`ReservedWords`] searches it by halving.
const GO_KEYWORDS: [&str; 80] = [
    "any",
    "append",
    "body",
    "bool",
    "break",
    "byte",
    "cap",
    "case",
    "chan",
    "clear",
    "close",
    "comparable",
    "complex",
    "complex128",
    "complex64",
    "const",
    "continue",
    "copy",
    "ctx",
    "default",
    "defer",
    "delete",
    "else",
    "err",
    "error",
    "failure",
    "fallthrough",
    "false",
    "float32",
    "float64",
    "for",
    "func",
    "go",
    "goto",
    "group",
    "if",
    "imag",
    "import",
    "int",
    "int16",
    "int32",
    "int64",
    "int8",
    "interface",
    "iota",
    "len",
    "make",
    "map",
    "max",
    "min",
    "new",
    "nil",
    "out",
    "package",
    "panic",
    "path",
    "payload",
    "print",
    "println",
    "query",
    "range",
    "real",
    "recover",
    "return",
    "rune",
    "select",
    "status",
    "string",
    "struct",
    "switch",
    "transport",
    "true",
    "type",
    "uint",
    "uint16",
    "uint32",
    "uint64",
    "uint8",
    "uintptr",
    "var",
];

/// What a name rendering to no identifier at all is spelled as in Go.
const GO_PLACEHOLDER: &str = "value";

static GO_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&GO_KEYWORDS, Escape::Suffix('_'), GO_PLACEHOLDER).expect(
        "the Go keyword table is sorted, carries no empty word, and does not hold its own fallback",
    )
});

/// How Go wants what it is handed spelled.
///
/// The casing of the first letter is what decides whether a name leaves its package, so everything
/// a caller reaches — types, methods, fields, constants — is [`Case::UpperCamel`], and only the
/// arguments of a method, which never leave the body they are declared in, are lower.
fn go() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::UpperCamel,
            field: Case::UpperCamel,
            parameter: Case::LowerCamel,
            constant: Case::UpperCamel,
            file: Case::Snake,
            module: Case::Lower,
        },
        reserved: LazyLock::force(&GO_RESERVED),
        scalars: ScalarNames {
            string: "string",
            // Neither of these two names a type the standard library carries, so the target
            // declares them; `time.Time` and the rest are Go's own.
            uuid: "UUID",
            date_time: "time.Time",
            date: "Date",
            url: "string",
            integer32: "int32",
            integer64: "int64",
            number: "float64",
            boolean: "bool",
        },
        comment: CommentStyle::DoubleSlash,
        extension: "go",
    }
}

/// The words TypeScript keeps for itself where this target puts a name, and the names the emitter
/// writes into every method body.
///
/// Only the words a *binding* may not be spelled as are here: a name that is merely contextual —
/// `from`, `get`, `type`, `string` — is a name an argument may carry, and spelling it out of the
/// way would put an underscore in a published signature for nothing. `constructor` is the one
/// exception in the other direction: a class declares its own initialiser under that name.
///
/// The emitter's own names are here because an argument spelling `path` would be assigned over by
/// the line that fills in the path template, and one spelling `queryValue` would be called by the
/// line that writes a parameter into the query. Sorted, as [`ReservedWords`] searches it by
/// halving.
const TYPESCRIPT_KEYWORDS: [&str; 59] = [
    "await",
    "body",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "constructor",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "issued",
    "let",
    "new",
    "null",
    "package",
    "path",
    "pathSegment",
    "payload",
    "private",
    "protected",
    "public",
    "query",
    "queryValue",
    "raiseForStatus",
    "read",
    "readPayload",
    "return",
    "static",
    "status",
    "super",
    "switch",
    "this",
    "throw",
    "transport",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// What a name rendering to no identifier at all is spelled as in TypeScript.
const TYPESCRIPT_PLACEHOLDER: &str = "value";

static TYPESCRIPT_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(
        &TYPESCRIPT_KEYWORDS,
        Escape::Suffix('_'),
        TYPESCRIPT_PLACEHOLDER,
    )
    .expect(
        "the TypeScript keyword table is sorted, carries no empty word, and does not hold its own \
         fallback",
    )
});

/// The names a TypeScript method may not carry, even though a binding may carry every one of them.
///
/// None of these is a keyword — that table is the other one, and it does not apply where a method
/// sits. These are names every object already answers to: the members the prototype carries, and
/// the one that makes an object look like a promise. A method named `then` turns the group it sits
/// on into something `await` unwraps, and a method named `toString` decides what the group reads as
/// the next time anything prints it — both of them a long way from the declaration that caused it.
///
/// Only what a name can actually render to is listed: rendering lowercases what follows the first
/// letter of a word, so `to.json` reaches `toJson` and never the `toJSON` a serialiser looks for.
/// Sorted, as [`ReservedWords`] searches it by halving.
const TYPESCRIPT_SHADOWED: [&str; 8] = [
    "constructor",
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
    "then",
    "toLocaleString",
    "toString",
    "valueOf",
];

static TYPESCRIPT_METHOD_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(
        &TYPESCRIPT_SHADOWED,
        Escape::Suffix('_'),
        TYPESCRIPT_PLACEHOLDER,
    )
    .expect(
        "the TypeScript shadowed table is sorted, carries no empty word, and does not hold its own \
         fallback",
    )
});

/// What a TypeScript method may not be named after.
///
/// A method sits where a member sits, and every word the language keeps for itself is a name a
/// member may carry, so the keyword table does not apply there: an operation the document calls
/// `delete` is a method called `delete`. What does apply is what every object already answers to.
pub(super) fn typescript_method_reserved() -> &'static ReservedWords {
    LazyLock::force(&TYPESCRIPT_METHOD_RESERVED)
}

/// How TypeScript wants what it is handed spelled.
///
/// Every scalar the document states as text is text here: what travels in JSON is a string, and a
/// type that stood for anything else would need a conversion between what was parsed and what the
/// type claims, which is a layer this target deliberately does not have.
fn typescript() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::LowerCamel,
            field: Case::LowerCamel,
            parameter: Case::LowerCamel,
            // How the members of a closed set of values are named, which is not how a module-level
            // constant is named: the members read as the values they stand for.
            constant: Case::UpperCamel,
            file: Case::LowerCamel,
            module: Case::LowerCamel,
        },
        reserved: LazyLock::force(&TYPESCRIPT_RESERVED),
        scalars: ScalarNames {
            string: "string",
            uuid: "string",
            date_time: "string",
            date: "string",
            url: "string",
            integer32: "number",
            integer64: "number",
            number: "number",
            boolean: "boolean",
        },
        comment: CommentStyle::DoubleSlash,
        extension: "ts",
    }
}

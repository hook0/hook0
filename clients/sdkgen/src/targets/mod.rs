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

pub mod csharp;
pub mod go;
pub mod java;
pub mod kotlin;
pub mod lua;
pub mod mcp;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod typescript;
pub mod zig;

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

/// Which of the shared corpus a target's client is a contract for.
///
/// The corpus at `clients/conformance` is a set of documents about the things a client does: what
/// it puts on the wire, what it repeats, what it verifies, what it bounds. Which of them hold a
/// given target up is a property of that target, so it is stated here rather than guessed from
/// something else — an SDK owning a whole tree happens to do all four today, and reading the
/// contract off that coincidence held the MCP client to none of them while it was quietly dropping
/// two of the headers every other client sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contract {
    /// Every document the corpus holds, whatever it comes to hold: a client that sends, repeats,
    /// verifies and bounds is held to all of it, and to a document added tomorrow without this
    /// entry being touched.
    Whole,
    /// Only the documents named, which is the whole of what this target's client takes part in. A
    /// document it cannot honour would make the guard wrong rather than strict, and a document it
    /// can is one nothing else would hold it to.
    Only(&'static [&'static str]),
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
    /// What of the shared corpus the client this target lands in is held to.
    pub contract: Contract,
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
        csharp::target(),
        go::target(),
        java::target(),
        kotlin::target(),
        lua::target(),
        mcp::target(),
        php::target(),
        python::target(),
        ruby::target(),
        rust::target(),
        typescript::target(),
        zig::target(),
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

/// Everything a name emitted as Java has to stay out of the way of.
///
/// Three sets, merged because in Java they apply in the same places rather than in different ones.
///
/// The language's own words — its fifty keywords, the `_` it took back in Java 9, and the three
/// literals — are illegal wherever an identifier goes: as a field, as a method, as an argument and
/// as a type alike. That is the opposite of Ruby, where a keyword is a perfectly good method name,
/// and it is why there is one table here rather than two.
///
/// The eight no-argument methods of `java.lang.Object` are the second set. They are legal as a
/// field and as an argument, and illegal as a method — and a `record` component is *both*: it
/// declares a field and the accessor that reads it, so §8.10.3 refuses a component spelled after
/// any of them outright. Every value this target declares is a record, so they belong here. What is
/// deliberately absent is `equals`: it takes an argument, so a no-argument `equals()` overloads it
/// rather than replacing it, and the compiler accepts a component of that name.
///
/// The last set is the emitter's own locals. An operation whose parameter is spelled `path` would
/// be declared twice in the method that fills the path template, and one spelled `out` would shadow
/// the map a value is written into — the first fails to compile and the second writes the wrong
/// document. Go and TypeScript keep their locals here for the same reason.
///
/// The restricted type identifiers — `record`, `sealed`, `permits`, `var`, `yield` — are absent on
/// purpose: they are refused only where a *type* is named, and a type name is rendered
/// [`Case::UpperCamel`] here, so no name this target declares can ever spell one of them. Every
/// entry below was measured against `javac` rather than read off a list, and the measurement is
/// committed as a case of the Java suite so it stays a fact.
///
/// Sorted, as [`ReservedWords`] searches it by halving.
const JAVA_KEYWORDS: [&str; 69] = [
    "_",
    "abstract",
    "answered",
    "assert",
    "body",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "clone",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "false",
    "fields",
    "final",
    "finalize",
    "finally",
    "float",
    "for",
    "getClass",
    "goto",
    "hashCode",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "notify",
    "notifyAll",
    "null",
    "out",
    "package",
    "path",
    "private",
    "protected",
    "public",
    "query",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "toString",
    "transient",
    "transport",
    "true",
    "try",
    "void",
    "volatile",
    "wait",
    "while",
];

/// What a name rendering to no identifier at all is spelled as in Java.
const JAVA_PLACEHOLDER: &str = "value";

static JAVA_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&JAVA_KEYWORDS, Escape::Suffix('_'), JAVA_PLACEHOLDER).expect(
        "the Java keyword table is sorted, carries no empty word, and does not hold its own fallback",
    )
});

/// How Java wants what it is handed spelled.
///
/// Every number is the boxed type rather than the primitive one: a member the document does not
/// require is absent as `null`, and `int` has no way to be absent. A file is named after the type
/// it declares — the language insists on it — so files carry the casing type names do.
fn java() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::LowerCamel,
            field: Case::LowerCamel,
            parameter: Case::LowerCamel,
            constant: Case::ScreamingSnake,
            file: Case::UpperCamel,
            module: Case::Lower,
        },
        reserved: LazyLock::force(&JAVA_RESERVED),
        scalars: ScalarNames {
            string: "String",
            uuid: "UUID",
            date_time: "OffsetDateTime",
            date: "LocalDate",
            // Nothing in the standard library reads every URL the API can answer, and a type that
            // refused one would lose a value the document says is text.
            url: "String",
            integer32: "Integer",
            integer64: "Long",
            number: "Double",
            boolean: "Boolean",
        },
        comment: CommentStyle::DoubleSlash,
        extension: "java",
    }
}

/// Everything a name emitted as Kotlin has to stay out of the way of wherever a *binding* goes.
///
/// The language's twenty-eight hard keywords are illegal as a property, as an argument and as a
/// function alike, which was measured against `kotlinc` rather than read off a list: every one of
/// the three probes is refused for every one of them. What is deliberately absent is the soft and
/// modifier vocabulary — `data`, `value`, `field`, `get`, `set`, `open`, `sealed`, `suspend` and the
/// rest — because the parser reads those as names wherever this target puts one, and reserving them
/// would rename a member the API is entitled to declare for a collision the language does not have.
///
/// Backticks are the escape Kotlin offers for the twenty-eight, and they were measured too: they do
/// rescue all of them in all three positions. They are not used here, for two reasons. A backticked
/// property compiles to the getter its name dictates, so `val \`class\`` emits `getClass()` — and a
/// Java caller of the published artefact then reads the property where it wrote `getClass()`, which
/// was measured as well and is a silent wrong answer rather than a compilation failure. And a
/// backtick rescues nothing in the one position that actually needs rescuing below, so a
/// backtick-only strategy would still need a table. A suffix is what seven of the other targets
/// spell an escape as, and it reads the same from Kotlin and from Java.
///
/// The last five are the emitter's own names. An operation whose parameter is spelled `path` would
/// shadow the local that fills the path template, one spelled `transport` would shadow the property
/// every request is issued through, and a member spelled `out` would be written into the map that
/// is being built out of it. Java, Go and TypeScript keep their locals here for the same reason.
///
/// Sorted, as [`ReservedWords`] searches it by halving.
const KOTLIN_KEYWORDS: [&str; 33] = [
    "as",
    "body",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "out",
    "package",
    "path",
    "query",
    "return",
    "super",
    "this",
    "throw",
    "transport",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
];

/// The two names a declaration would *hide* rather than merely sit beside, and only in one position.
///
/// This is the positional distinction Kotlin has, and it is the opposite way round from Java's. In
/// Java the keywords are refused everywhere and the no-argument methods of `Object` are refused only
/// where a method is declared; in Kotlin the keywords are refused everywhere *and escapable*, while
/// these two are refused only where a function is declared and are not escapable at all —
/// `` fun `toString`() `` is the same declaration as `fun toString()` and draws the same refusal,
/// which was measured rather than assumed.
///
/// A property spelled either way is fine: Kotlin compiles `val hashCode: String` to `getHashCode()`,
/// so it sits beside `hashCode()` instead of replacing it, and a data class carrying one still
/// generates its own. `equals` is deliberately absent for the reason it is absent from the Java
/// table: a declaration of that name takes an argument of another type, so it overloads rather than
/// hides, and the compiler accepts it. `copy` and `component1` are absent because only a data class
/// generates those, and no data class this target writes declares a method beyond `toJson`.
///
/// Sorted, as [`ReservedWords`] searches it by halving.
const KOTLIN_SHADOWED: [&str; 2] = ["hashCode", "toString"];

/// What a name rendering to no identifier at all is spelled as in Kotlin.
const KOTLIN_PLACEHOLDER: &str = "value";

static KOTLIN_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&KOTLIN_KEYWORDS, Escape::Suffix('_'), KOTLIN_PLACEHOLDER).expect(
        "the Kotlin keyword table is sorted, carries no empty word, and does not hold its own \
         fallback",
    )
});

/// What a *function* of an emitted Kotlin declaration has to stay out of the way of: the keywords,
/// which apply there as everywhere, together with the two names a function would hide. The two
/// halves are merged into the one sorted list [`ReservedWords`] searches, since a reader of one
/// declaration should not have to work out which of two tables applies where.
static KOTLIN_METHOD_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    let mut words: Vec<&str> = KOTLIN_KEYWORDS
        .iter()
        .chain(KOTLIN_SHADOWED.iter())
        .copied()
        .collect();
    words.sort_unstable();
    words.dedup();

    ReservedWords::build(&words, Escape::Suffix('_'), KOTLIN_PLACEHOLDER).expect(
        "the merged Kotlin vocabulary is sorted, carries no empty word, and does not hold its own \
         fallback",
    )
});

/// The vocabulary a function of an emitted Kotlin declaration is spelled out of the way of.
pub(super) fn kotlin_method_reserved() -> &'static ReservedWords {
    LazyLock::force(&KOTLIN_METHOD_RESERVED)
}

/// How Kotlin wants what it is handed spelled.
///
/// Every number is the type the language reads a JSON number back as; nothing is boxed by hand,
/// since absence is a property of the type here rather than of the class that carries it — a member
/// the document does not require is declared `T?` and every other one is not. A file is named after
/// the type it declares, which the language does not insist on and this target does anyway: the
/// whole directory is owned by the generator, so a type that stops being declared has to take a file
/// with it.
fn kotlin() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::LowerCamel,
            field: Case::LowerCamel,
            parameter: Case::LowerCamel,
            constant: Case::ScreamingSnake,
            file: Case::UpperCamel,
            module: Case::Lower,
        },
        reserved: LazyLock::force(&KOTLIN_RESERVED),
        scalars: ScalarNames {
            string: "String",
            uuid: "UUID",
            date_time: "OffsetDateTime",
            date: "LocalDate",
            // Nothing in the standard library reads every URL the API can answer, and a type that
            // refused one would lose a value the document says is text.
            url: "String",
            integer32: "Int",
            integer64: "Long",
            number: "Double",
            boolean: "Boolean",
        },
        comment: CommentStyle::DoubleSlash,
        extension: "kt",
    }
}

/// Lua's own vocabulary: the twenty-two words the language keeps for itself.
///
/// Every one of them is illegal wherever an identifier goes, and — unlike Ruby or PHP — that
/// includes the position a member sits in: `t.end` is a parse error, and the only way to reach a
/// member spelled that way is `t["end"]`, which is not what a caller of a generated SDK should have
/// to write. So there is one vocabulary here rather than two, and it applies to members, to methods
/// and to arguments alike.
const LUA_KEYWORDS: [&str; 22] = [
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

/// The names an emitted declaration would *replace* rather than merely sit beside.
///
/// Three things are in here. The metamethods are what a table already answers to through its
/// metatable, and a member carrying one of them under a different meaning changes what happens a
/// long way from where it was declared: a `__eq` that is a value the API sent decides what two
/// documents comparing equal means, and a `__index` that is a member turns every missing lookup
/// into whatever the API happened to answer. None of them can be reached by a name the document
/// spells today — rendering drops the leading underscores, so `__index` arrives as `index` — but
/// they are the language's vocabulary rather than today's surface, and this table describes the
/// language.
///
/// The second set is what every emitted value already carries: the constructor, the two decoders,
/// the writer and the membership test. A member of an instance shadows what the table it was built
/// from declares, so a field spelled `to_table` is a string where a method should be, and the value
/// carrying it can no longer be written back.
///
/// The third is the names the emitter itself writes into the file a group sits in: the two helpers
/// every operation body calls, which a parameter of that name would shadow inside the very method
/// that calls them, and `self`, which a method declared with a colon receives whether or not it
/// asked for one — a parameter spelled that way is declared twice in one signature, which the
/// language refuses outright.
///
/// What is deliberately absent is `transport`. A group holds one, and every emitted method reads it
/// as `self.transport` rather than as a bare name, so an operation declaring a parameter of that
/// name shadows nothing: reserving it would rename an argument the API is entitled to declare for a
/// collision that cannot happen.
///
/// Sorted, as [`ReservedWords`] searches it by halving — which puts the capitalised member first
/// and the underscored ones ahead of the rest.
const LUA_SHADOWED: [&str; 37] = [
    "VALUES",
    "__add",
    "__band",
    "__bnot",
    "__bor",
    "__bxor",
    "__call",
    "__close",
    "__concat",
    "__div",
    "__eq",
    "__gc",
    "__idiv",
    "__index",
    "__le",
    "__len",
    "__lt",
    "__metatable",
    "__mod",
    "__mode",
    "__mul",
    "__name",
    "__newindex",
    "__pairs",
    "__pow",
    "__shl",
    "__shr",
    "__sub",
    "__tostring",
    "__unm",
    "check_answer",
    "from_json",
    "member",
    "new",
    "read_answer",
    "self",
    "to_table",
];

/// What a name rendering to no identifier at all is spelled as in Lua.
const LUA_PLACEHOLDER: &str = "value";

/// Everything a name emitted as Lua has to stay out of the way of, merged into the one sorted list
/// [`ReservedWords`] searches.
static LUA_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    let mut words: Vec<&str> = LUA_KEYWORDS
        .iter()
        .chain(LUA_SHADOWED.iter())
        .copied()
        .collect();
    words.sort_unstable();
    words.dedup();

    ReservedWords::build(&words, Escape::Suffix('_'), LUA_PLACEHOLDER).expect(
        "the merged Lua vocabulary is sorted, carries no empty word, and does not hold its own \
         fallback",
    )
});

/// How Lua wants what it is handed spelled.
///
/// Every scalar the document states as text is text here, and so are the three the document gives a
/// format to: the language carries no type for an identifier, a moment or a day, and what the API
/// answered is what has to go back out unchanged. A whole number is `integer` rather than `number`
/// because Lua 5.3 split the two, and writing a count back as `1.0` is a document the API refuses.
///
/// A file is named after the layer it carries rather than after a type, since the rockspec is what
/// maps a module name onto a path.
fn lua() -> LanguageSpec {
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
        reserved: LazyLock::force(&LUA_RESERVED),
        scalars: ScalarNames {
            string: "string",
            uuid: "string",
            date_time: "string",
            date: "string",
            url: "string",
            integer32: "integer",
            integer64: "integer",
            number: "number",
            boolean: "boolean",
        },
        comment: CommentStyle::DoubleDash,
        extension: "lua",
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

/// The words C# keeps for itself where this target puts a *binding*.
///
/// Every one of them is escapable — `@class` is a name the compiler reads as `class` — and that
/// escape is invisible to everything else: `@class` and `class` are the same identifier, so the
/// prefix moves a name out of the parser's way and out of nothing else's. That is why this table is
/// the keywords alone, and why the shadowing half below is escaped differently.
///
/// Only the reserved words are listed. A contextual one — `value`, `var`, `record`, `async`,
/// `await`, `nameof` — is a name a binding may carry, and spelling it out of the way would put an
/// `@` in a published signature for a collision the language does not have.
///
/// Sorted, as [`ReservedWords`] searches it by halving.
const CSHARP_KEYWORDS: [&str; 77] = [
    "abstract",
    "as",
    "base",
    "bool",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "checked",
    "class",
    "const",
    "continue",
    "decimal",
    "default",
    "delegate",
    "do",
    "double",
    "else",
    "enum",
    "event",
    "explicit",
    "extern",
    "false",
    "finally",
    "fixed",
    "float",
    "for",
    "foreach",
    "goto",
    "if",
    "implicit",
    "in",
    "int",
    "interface",
    "internal",
    "is",
    "lock",
    "long",
    "namespace",
    "new",
    "null",
    "object",
    "operator",
    "out",
    "override",
    "params",
    "private",
    "protected",
    "public",
    "readonly",
    "ref",
    "return",
    "sbyte",
    "sealed",
    "short",
    "sizeof",
    "stackalloc",
    "static",
    "string",
    "struct",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "uint",
    "ulong",
    "unchecked",
    "unsafe",
    "ushort",
    "using",
    "virtual",
    "void",
    "volatile",
    "while",
];

/// The names an emitted *member* would collide with rather than merely sit beside.
///
/// These are what every object already answers to, and what the compiler says about each of them
/// was measured rather than reasoned about. A property spelled `Equals`, `GetHashCode` or
/// `ToString` is `error CS0102`: the declaration the compiler already wrote for the type is
/// there first. One spelled `GetType`, `MemberwiseClone` or `ReferenceEquals` is `warning CS0108`,
/// which a build that treats warnings as errors reads the same way. `Finalize` produces neither and
/// is therefore absent: reserving it would rename a member the API is entitled to declare for a
/// collision the compiler does not have.
///
/// The keyword table does not apply here at all, which is the positional distinction C# has.
/// A member is rendered in upper camel case and C# reserves nothing that is spelled that way, so a
/// field the document calls `class` is a property called `Class` and needs no escape. What a member
/// does have to avoid is this list — and, separately and per declaration, the name of the type it
/// sits in, which `error CS0542` refuses and which no fixed vocabulary can express.
///
/// Sorted, as [`ReservedWords`] searches it by halving.
const CSHARP_SHADOWED: [&str; 6] = [
    "Equals",
    "GetHashCode",
    "GetType",
    "MemberwiseClone",
    "ReferenceEquals",
    "ToString",
];

/// What a name rendering to no identifier at all is spelled as in C#, in each position.
const CSHARP_PLACEHOLDER: &str = "value";
const CSHARP_MEMBER_PLACEHOLDER: &str = "Value";

static CSHARP_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&CSHARP_KEYWORDS, Escape::Prefix('@'), CSHARP_PLACEHOLDER).expect(
        "the C# keyword table is sorted, carries no empty word, and does not hold its own fallback",
    )
});

/// What a member of an emitted C# declaration is spelled out of the way of.
///
/// The escape is a suffix rather than the `@` the keywords use, because `@` is not an escape at
/// all here: the compiler reads `@ToString` and `ToString` as one name, so prefixing it would
/// rename nothing and the collision would ship.
static CSHARP_MEMBER_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(
        &CSHARP_SHADOWED,
        Escape::Suffix('_'),
        CSHARP_MEMBER_PLACEHOLDER,
    )
    .expect(
        "the C# shadowing table is sorted, carries no empty word, and does not hold its own \
         fallback",
    )
});

/// The vocabulary a member of an emitted C# declaration is spelled out of the way of.
pub(super) fn csharp_member_reserved() -> &'static ReservedWords {
    LazyLock::force(&CSHARP_MEMBER_RESERVED)
}

/// How C# wants what it is handed spelled.
///
/// Everything a caller reaches is upper camel case, which is what the framework's own surface is
/// spelled as; only the arguments of a method, which never leave the signature they are declared
/// in, are lower. A file is named after the type it carries, so it is cased like one.
fn csharp() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::UpperCamel,
            field: Case::UpperCamel,
            parameter: Case::LowerCamel,
            constant: Case::UpperCamel,
            file: Case::UpperCamel,
            module: Case::UpperCamel,
        },
        reserved: LazyLock::force(&CSHARP_RESERVED),
        scalars: ScalarNames {
            string: "string",
            uuid: "Guid",
            date_time: "DateTimeOffset",
            date: "DateOnly",
            // What the API answers is text, and text is what has to go back out unchanged; `Uri`
            // would put a normalisation nobody asked for between the two.
            url: "string",
            integer32: "int",
            integer64: "long",
            number: "double",
            boolean: "bool",
        },
        comment: CommentStyle::DoubleSlash,
        extension: "cs",
    }
}

/// The words PHP keeps for itself where this target puts a *type* name, written the way a type name
/// reaches them.
///
/// PHP compares its own vocabulary without regard to case — `class List` and `class LIST` are the
/// same parse error — and every name this target spells reaches a table under exactly one
/// [`Case`]. A type name is upper camel case, so the words are written that way and compared
/// exactly: a table written in lower case would be searched for `List` and find nothing, and the
/// keyword would ship as a class the language refuses to parse.
///
/// The list is the language's reserved words together with the names it refuses as a class outright
/// (`Object`, `Int`, `Never` and the rest), since both fail the same way. What is deliberately
/// absent is every keyword spelled with a separator — `include_once`, `require_once`,
/// `__halt_compiler` — because rendering drops the separator and `IncludeOnce` is a class name the
/// language accepts. Sorted, as [`ReservedWords`] searches it by halving.
const PHP_KEYWORDS: [&str; 84] = [
    "Abstract",
    "And",
    "Array",
    "As",
    "Bool",
    "Break",
    "Callable",
    "Case",
    "Catch",
    "Class",
    "Clone",
    "Const",
    "Continue",
    "Declare",
    "Default",
    "Die",
    "Do",
    "Echo",
    "Else",
    "Elseif",
    "Empty",
    "Enddeclare",
    "Endfor",
    "Endforeach",
    "Endif",
    "Endswitch",
    "Endwhile",
    "Enum",
    "Eval",
    "Exit",
    "Extends",
    "False",
    "Final",
    "Finally",
    "Float",
    "Fn",
    "For",
    "Foreach",
    "Function",
    "Global",
    "Goto",
    "If",
    "Implements",
    "Include",
    "Instanceof",
    "Insteadof",
    "Int",
    "Interface",
    "Isset",
    "Iterable",
    "List",
    "Match",
    "Mixed",
    "Namespace",
    "Never",
    "New",
    "Null",
    "Numeric",
    "Object",
    "Or",
    "Parent",
    "Print",
    "Private",
    "Protected",
    "Public",
    "Readonly",
    "Require",
    "Resource",
    "Return",
    "Self",
    "Static",
    "String",
    "Switch",
    "Throw",
    "Trait",
    "True",
    "Try",
    "Unset",
    "Use",
    "Var",
    "Void",
    "While",
    "Xor",
    "Yield",
];

/// The names a member of an emitted PHP declaration would *replace* rather than merely sit beside.
///
/// Three things are in here, and the language's keywords are not, on purpose: PHP reads what
/// follows `function`, `->`, `::` and `$` as a name rather than as a keyword, so a method called
/// `list`, a property called `class` and an argument called `print` are all names a caller can
/// write, and escaping them would put an underscore in a published package for a collision that
/// cannot happen.
///
/// `Class` is the one word the language does refuse in a member position: a class constant — and
/// therefore an enumeration case — may not be spelled that way, because `Something::class` already
/// names the class itself. It is upper camel case here, which is how an enumeration case reaches
/// this table.
///
/// The magic methods are what every class already answers to, and a declaration carrying one of
/// them under a different meaning changes what happens a long way from where it was declared: a
/// `__toString` decides what the value reads as the next time anything prints it. Rendering drops
/// the leading underscores, so no name the document carries reaches one today; they are the
/// language's vocabulary rather than today's surface, and the table describes the language.
///
/// The rest are the names the emitter itself writes into every declaration — the two helpers an
/// operation group calls, the two the base exception calls, and the three every emitted type
/// carries — which a member spelled the same way would take the place of. Sorted, as
/// [`ReservedWords`] searches it by halving, which puts the upper-case word first and the
/// underscored ones ahead of the rest.
const PHP_SHADOWED: [&str; 25] = [
    "Class",
    "__call",
    "__callStatic",
    "__clone",
    "__construct",
    "__debugInfo",
    "__destruct",
    "__get",
    "__invoke",
    "__isset",
    "__serialize",
    "__set",
    "__set_state",
    "__sleep",
    "__toString",
    "__unserialize",
    "__unset",
    "__wakeup",
    "checkAnswer",
    "equals",
    "fromJson",
    "problemOf",
    "raiseForStatus",
    "readAnswer",
    "toArray",
];

/// What a name rendering to no identifier at all is spelled as in PHP.
const PHP_PLACEHOLDER: &str = "value";

static PHP_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&PHP_KEYWORDS, Escape::Suffix('_'), PHP_PLACEHOLDER).expect(
        "the PHP keyword table is sorted, carries no empty word, and does not hold its own fallback",
    )
});

static PHP_SHADOWED_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&PHP_SHADOWED, Escape::Suffix('_'), PHP_PLACEHOLDER).expect(
        "the PHP shadowing table is sorted, carries no empty word, and does not hold its own \
         fallback",
    )
});

/// The vocabulary a member of an emitted PHP declaration is spelled out of the way of.
pub(super) fn php_shadowed() -> &'static ReservedWords {
    LazyLock::force(&PHP_SHADOWED_RESERVED)
}

/// How PHP wants what it is handed spelled.
///
/// A file is named after the type it carries, since that is what an autoloader turns a class name
/// back into, so it is cased like one. An enumeration case is upper camel case, which is what the
/// language's own style guides spell one as, and is also what keeps it clear of `class` being the
/// one member name the language refuses.
///
/// Every identifier the API answers travels as the text it answered: PHP has no type for one, and
/// that text is what has to go back out unchanged. A day and a moment share the one immutable type
/// the language carries for both, which is why how much of either survives being written is decided
/// in the runtime rather than read off the type.
fn php() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::LowerCamel,
            field: Case::LowerCamel,
            parameter: Case::LowerCamel,
            constant: Case::UpperCamel,
            file: Case::UpperCamel,
            module: Case::UpperCamel,
        },
        reserved: LazyLock::force(&PHP_RESERVED),
        scalars: ScalarNames {
            string: "string",
            uuid: "string",
            date_time: "\\DateTimeImmutable",
            date: "\\DateTimeImmutable",
            url: "string",
            integer32: "int",
            integer64: "int",
            number: "float",
            boolean: "bool",
        },
        comment: CommentStyle::DoubleSlash,
        extension: "php",
    }
}

/// Zig's own vocabulary, plus the names the emitter writes into every method body.
///
/// The keywords are illegal wherever an identifier goes, a struct field included: `error: []const
/// u8` is a parse error, and so is a parameter spelled `align`. Zig does have an escape that would
/// admit them — `@"error"` is a legal identifier, and it is what a hand-written client would reach
/// for — but [`Escape`] carries one character, a prefix or a suffix, and quoting is neither. This
/// target therefore spells a keyword out of the way with `_` as eight of the other targets do, which
/// costs a name the API is entitled to declare an underscore and costs nobody a compile.
///
/// The rest are what an argument would shadow inside the very body that reads it. Every emitted
/// method declares an allocator, an arena, the answer it read back and — when it answers a value —
/// the `Owned` holding it; `self` and `fields` are the ones the language and the decoder put there.
/// A parameter carrying one of those names would be read in place of what the emitter meant.
///
/// The primitive type names are deliberately absent. `u8`, `bool` and the rest are not keywords, and
/// a struct field or an argument may carry any of them: what would be shadowed is the type inside
/// that one signature, and no emitted signature names a primitive after the argument that shadows
/// it. Reserving them would rename a field the document is entitled to declare for a collision the
/// language does not have.
///
/// Sorted, as [`ReservedWords`] searches it by halving.
const ZIG_KEYWORDS: [&str; 62] = [
    "addrspace",
    "align",
    "allocator",
    "allowzero",
    "and",
    "answered",
    "anyframe",
    "anytype",
    "arena",
    "asm",
    "async",
    "await",
    "break",
    "callconv",
    "catch",
    "comptime",
    "const",
    "continue",
    "defer",
    "else",
    "enum",
    "errdefer",
    "error",
    "export",
    "extern",
    "fields",
    "fn",
    "for",
    "fromJson",
    "held",
    "if",
    "inline",
    "linksection",
    "noalias",
    "noinline",
    "nosuspend",
    "opaque",
    "or",
    "orelse",
    "out",
    "owned",
    "packed",
    "pub",
    "reported",
    "resume",
    "return",
    "self",
    "struct",
    "suspend",
    "switch",
    "test",
    "threadlocal",
    "toJson",
    "transport",
    "try",
    "union",
    "unreachable",
    "usingnamespace",
    "value",
    "var",
    "volatile",
    "while",
];

/// What a name rendering to no identifier at all is spelled as in Zig.
///
/// Not the `value` every other target falls back on: that name is already taken here, by the one
/// argument every decoder is handed.
const ZIG_PLACEHOLDER: &str = "member";

static ZIG_RESERVED: LazyLock<ReservedWords> = LazyLock::new(|| {
    ReservedWords::build(&ZIG_KEYWORDS, Escape::Suffix('_'), ZIG_PLACEHOLDER).expect(
        "the Zig keyword table is sorted, carries no empty word, and does not hold its own fallback",
    )
});

/// How Zig wants what it is handed spelled.
///
/// Every scalar the document states as text is text here, and so are the three the document gives a
/// format to: the language carries no type for an identifier, a moment or a day, and what the API
/// answered is what has to go back out unchanged — a type that stood for anything else would need a
/// conversion between what was parsed and what the type claims, and would have to allocate to make
/// it.
///
/// A file is named after the layer it carries rather than after a type, since a Zig file is a struct
/// and one file may declare as many as it likes.
fn zig() -> LanguageSpec {
    LanguageSpec {
        casing: Casing {
            type_name: Case::UpperCamel,
            method: Case::LowerCamel,
            field: Case::Snake,
            parameter: Case::Snake,
            constant: Case::Snake,
            file: Case::Snake,
            module: Case::Snake,
        },
        reserved: LazyLock::force(&ZIG_RESERVED),
        scalars: ScalarNames {
            string: "[]const u8",
            uuid: "[]const u8",
            date_time: "[]const u8",
            date: "[]const u8",
            url: "[]const u8",
            integer32: "i32",
            integer64: "i64",
            number: "f64",
            boolean: "bool",
        },
        comment: CommentStyle::DoubleSlash,
        extension: "zig",
    }
}

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

pub mod mcp;

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

static TARGETS: LazyLock<Vec<Target>> = LazyLock::new(|| vec![mcp::target()]);

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

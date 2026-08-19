//! Shapes the names a target writes, and keeps them out of the language's own vocabulary.
//!
//! A name reaches a target as whatever the snapshot spells — `applications.list`, `event_type`,
//! `subscriptionId` — and every language wants it a different way. Splitting a name into the words
//! it is built from happens once, here, and each casing is rendered from those words rather than
//! from a name-mangling routine written again per target.
//!
//! Escaping travels with the vocabulary it guards. Rust spells an escaped `type` as `type_` and C#
//! spells it `@type`, so a caller names the vocabulary that applies and never how to get out of it.
//! [`escape`] answers for every name it is handed. One that rendered to nothing comes back as the
//! vocabulary's fallback, one that collides with a keyword comes back out of its way, and one that
//! is no identifier at all is refused rather than passed through. It used to be passed through, and
//! a property named `2fa` became `pub 2fa: String` in eleven languages that compile no such thing.

use crate::error::{Error, preview};
use crate::limits::Limits;

/// How the words of a name are spelled out as one identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Case {
    /// `event_type`
    Snake,
    /// `eventType`
    LowerCamel,
    /// `EventType`
    UpperCamel,
    /// `EVENT_TYPE`
    ScreamingSnake,
    /// `event-type`
    Kebab,
    /// `eventtype`, which is what a Go package or a Java package fragment looks like.
    Lower,
}

/// How a language spells an identifier that would otherwise be one of its keywords.
///
/// The marker is a character rather than a string, so an escape always grows the identifier it is
/// applied to — which is what lets [`escape`] step again when the escaped name is a keyword too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Escape {
    /// `type` becomes `type_`.
    Suffix(char),
    /// `type` becomes `@type`.
    Prefix(char),
}

impl Escape {
    fn applied_to(self, identifier: &str) -> String {
        match self {
            Self::Suffix(marker) => format!("{identifier}{marker}"),
            Self::Prefix(marker) => format!("{marker}{identifier}"),
        }
    }
}

/// The words a language keeps for itself, how it spells an identifier out of their way, and what it
/// falls back on when there is no name left to spell.
///
/// The list is held sorted and deduplicated so it can be searched by halving, and
/// [`ReservedWords::build`] refuses one that is not: an entry sorted out of place sits past the
/// point a search gives up on, and the keyword it names would go through unescaped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservedWords {
    words: Vec<String>,
    escape: Escape,
    placeholder: String,
}

impl ReservedWords {
    pub fn build(words: &[&str], escape: Escape, placeholder: &str) -> Result<Self, Error> {
        if placeholder.is_empty() {
            return Err(unusable(
                "it names nothing to fall back on when a name renders to no identifier".to_owned(),
            ));
        }

        let mut previous: Option<&str> = None;
        for word in words {
            if word.is_empty() {
                return Err(unusable(
                    "it carries a word spelling nothing, which no identifier can collide with"
                        .to_owned(),
                ));
            }
            if let Some(previous) = previous
                && *word <= previous
            {
                return Err(unusable(format!(
                    "it is searched in order, and `{}` does not sort after `{}`",
                    preview(word),
                    preview(previous)
                )));
            }
            previous = Some(word);
        }

        if words.contains(&placeholder) {
            return Err(unusable(format!(
                "its fallback `{}` is a keyword too, so a name rendering to no identifier would \
                 land on one",
                preview(placeholder)
            )));
        }

        Ok(Self {
            words: words.iter().map(|word| (*word).to_owned()).collect(),
            escape,
            placeholder: placeholder.to_owned(),
        })
    }

    /// Whether the language keeps this word for itself.
    pub fn contains(&self, candidate: &str) -> bool {
        self.words
            .binary_search_by(|word| word.as_str().cmp(candidate))
            .is_ok()
    }
}

/// Which casing a language wants for each kind of name it is given.
///
/// A target states this once instead of picking a [`Case`] at every call site, which is what keeps
/// two emitters of the same language from disagreeing on how a field is spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Casing {
    pub type_name: Case,
    pub method: Case,
    pub field: Case,
    /// How an argument of a method is spelled, which is not always how a member of a type is: a
    /// language where the casing of a name decides whether it leaves the package spells the two
    /// differently, and a language that reads them alike loses nothing by saying so twice.
    pub parameter: Case,
    pub constant: Case,
    pub file: Case,
    pub module: Case,
}

/// Splits a name into the words it is built from, across the separators the ecosystem uses (`.`,
/// `_`, `-`, a space, and camel-case humps).
///
/// A run of capitals is one word — `HTTPServer` reads as `httpserver` — because nothing in the name
/// says where such a run ends.
pub fn words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_was_lower = false;

    for character in name.chars() {
        if character == '.' || character == '_' || character == '-' || character == ' ' {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            previous_was_lower = false;
            continue;
        }

        if character.is_uppercase() && previous_was_lower && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }

        previous_was_lower = character.is_lowercase() || character.is_numeric();
        current.extend(character.to_lowercase());
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// [`words`], with the ceilings a name read off a snapshot has to sit under.
///
/// Everything downstream is bounded by these two: a name this accepts renders, under any [`Case`],
/// to an identifier no longer than the name itself plus one separator per word.
pub fn checked_words(name: &str, limits: &Limits) -> Result<Vec<String>, Error> {
    if name.len() > limits.max_identifier_bytes {
        return Err(Error::IdentifierTooLong {
            identifier: preview(name),
            size: name.len(),
            limit: limits.max_identifier_bytes,
        });
    }

    let words = words(name);
    if words.len() > limits.max_words_per_identifier {
        return Err(Error::TooManyWords {
            identifier: preview(name),
            count: words.len(),
            limit: limits.max_words_per_identifier,
        });
    }

    Ok(words)
}

/// Spells the words out as one identifier of the given casing, whatever case they arrived in.
///
/// Words spelling nothing contribute nothing, so a stray separator in the name never becomes a
/// stray separator in the identifier. A list that spells nothing at all renders to nothing, which
/// [`escape`] is what turns back into a usable name.
pub fn render(words: &[String], case: Case) -> String {
    let mut rendered = String::new();

    for word in words.iter().filter(|word| !word.is_empty()) {
        if !rendered.is_empty() {
            match case {
                Case::Snake | Case::ScreamingSnake => rendered.push('_'),
                Case::Kebab => rendered.push('-'),
                Case::LowerCamel | Case::UpperCamel | Case::Lower => {}
            }
        }

        match case {
            Case::Snake | Case::Kebab | Case::Lower => lowercased(&mut rendered, word),
            Case::ScreamingSnake => rendered.extend(word.chars().flat_map(char::to_uppercase)),
            Case::UpperCamel => capitalized(&mut rendered, word),
            Case::LowerCamel if rendered.is_empty() => lowercased(&mut rendered, word),
            Case::LowerCamel => capitalized(&mut rendered, word),
        }
    }

    rendered
}

/// The name that text is spelled under in one language: split into words, rendered under the
/// casing asked for, and moved out of the way of the language's own vocabulary.
///
/// Every target needs exactly this, and a target writing its own would be a second place where a
/// name could come out spelled differently from everywhere else.
pub fn spell(
    text: &str,
    case: Case,
    reserved: &ReservedWords,
    limits: &Limits,
) -> Result<String, Error> {
    let words = checked_words(text, limits)?;
    escape(&render(&words, case), reserved).map_err(|failure| match failure {
        // Re-raised against the name the snapshot spells, which is what somebody would have to
        // change, rather than against the rendering of it that this function happened to try.
        Error::UnspellableName {
            identifier, reason, ..
        } => Error::UnspellableName {
            name: preview(text),
            identifier,
            reason,
        },
        other => other,
    })
}

/// Spells a rendered identifier out of the way of a language's vocabulary, or says it cannot.
///
/// Escaping is applied again as long as it keeps landing on a keyword, which is what covers a
/// vocabulary holding both `type` and `type_`. Every application grows the identifier, so the
/// candidates are all distinct and the vocabulary runs out of words before the loop runs out of
/// steps.
///
/// A rendering that no language reads as an identifier is refused here rather than mangled into
/// one. Mangling would invent a name, and an invented name is published. It becomes the public
/// surface of twelve packages, and moving it afterwards is a breaking change to all of them. The
/// document is committed, so a refusal stops the generator on the commit that introduced the name,
/// where the fix belongs.
pub fn escape(rendered: &str, reserved: &ReservedWords) -> Result<String, Error> {
    if rendered.is_empty() {
        return Ok(reserved.placeholder.clone());
    }

    // A vocabulary that steps around a keyword by prefixing spells `@type`, and in the language
    // that does it that is a name. So the marker is allowed in front and what has to read as a
    // name is the rest, which is also what leaves an already-escaped identifier alone when it comes
    // back through here.
    let bare = match reserved.escape {
        Escape::Prefix(marker) => rendered.strip_prefix(marker).unwrap_or(rendered),
        Escape::Suffix(_) => rendered,
    };
    if let Some(reason) = unspellable(bare) {
        return Err(Error::UnspellableName {
            name: preview(rendered),
            identifier: preview(rendered),
            reason,
        });
    }

    let mut escaped = rendered.to_owned();
    for _ in 0..reserved.words.len() {
        if !reserved.contains(&escaped) {
            break;
        }
        escaped = reserved.escape.applied_to(&escaped);
    }

    Ok(escaped)
}

/// Why a rendering is no identifier, or nothing when it is one.
///
/// Held to ASCII rather than to what any one language allows. Go and C# read a letter from any
/// script, Zig and Lua read none, and one name is written into twelve languages at once, so what
/// all twelve read is the whole of what can be used.
///
/// A hyphen passes anywhere but first, because [`Case::Kebab`] is the only thing that can put one
/// there. [`words`] reads a hyphen as a separator, so no word carries one, and what kebab spells is
/// a file name or a package fragment rather than a name in the source.
fn unspellable(rendered: &str) -> Option<String> {
    let mut characters = rendered.chars();
    let Some(first) = characters.next() else {
        // Reached by a rendering that was nothing but an escape marker. The empty rendering itself
        // never arrives here, having already come back as the vocabulary's fallback.
        return Some("it spells nothing at all".to_owned());
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return Some(format!(
            "it opens on `{first}`, where an identifier opens on an ASCII letter or an underscore"
        ));
    }

    let stray = characters.find(|character| {
        !character.is_ascii_alphanumeric() && *character != '_' && *character != '-'
    })?;
    Some(format!(
        "it carries `{stray}`, where an identifier carries ASCII letters, digits and underscores"
    ))
}

fn lowercased(rendered: &mut String, word: &str) {
    rendered.extend(word.chars().flat_map(char::to_lowercase));
}

fn capitalized(rendered: &mut String, word: &str) {
    let mut characters = word.chars();
    if let Some(first) = characters.next() {
        rendered.extend(first.to_uppercase());
    }
    for character in characters {
        rendered.extend(character.to_lowercase());
    }
}

fn unusable(reason: String) -> Error {
    Error::UnusableReservedWords { reason }
}

//! What the dashboard shows under "Send an event", taken from the SDKs it shows it for.
//!
//! The screen used to carry its snippets as strings written by hand in a `.vue` file. One of them
//! did not compile: it passed a `&None` where the client declares an `Option<&Uuid>`, and had done
//! for as long as anybody could tell, because a snippet written by hand is backed by nothing. This
//! crate is the answer to that — every snippet the dashboard shows is a region of a file the SDK's
//! own job compiles, so a renamed method turns that job red on the day it happens rather than
//! turning up in somebody's editor weeks later.
//!
//! Nothing here is a list. Which SDKs the dashboard shows is a property the generator's registry
//! declares — the targets held to the whole conformance corpus — so the next client is shown the
//! day it is written and this file is not touched. What installs one comes from the release
//! inventory, which reads it out of the tree. What a language cannot be asked of a substitution is
//! declared in a manifest beside its examples.
//!
//! The guard is strict in both directions and has no list of exceptions: a target held to the whole
//! corpus without its three files stops the build, and examples sitting where no such target is
//! stop it too.

pub mod emit;
pub mod error;
pub mod install;
pub mod limits;
pub mod manifest;
pub mod region;

use std::path::{Path, PathBuf};

use hook0_sdkgen::targets::{Contract, targets};
use release_packages::{TargetRoot, Version};

pub use error::Error;
pub use manifest::StringLiteral;

/// The artefact this crate writes, relative to the repository.
pub const ARTEFACT: &str = "frontend/src/generated/sdkExamples.ts";

/// What rewrites it.
pub const REGENERATE: &str = "cargo run -p hook0-dashboard-examples";

/// What holds it to the examples it was written from.
pub const GUARD: &str = "cargo test -p hook0-dashboard-examples";

/// The directory every client occupies one of.
const CLIENTS: &str = "clients";

/// Where a client keeps what the dashboard shows.
const EXAMPLES: &str = "examples";

/// The example that sends an event, without its extension.
const SEND: &str = "dashboard_send";

/// The example that verifies a delivery, without its extension.
const VERIFY: &str = "dashboard_verify";

/// What the dashboard substitutes a form's value for.
///
/// Every one of them is a string literal, which is what lets a file full of them compile: a
/// `Uuid::parse_str("__HOOK0_APPLICATION_ID__")` builds, and would only fail if it were run, which
/// an example never is.
pub const API_URL: &str = "__HOOK0_API_URL__";
pub const APPLICATION_ID: &str = "__HOOK0_APPLICATION_ID__";
pub const TOKEN: &str = "__HOOK0_TOKEN__";
pub const EVENT_TYPE: &str = "__HOOK0_EVENT_TYPE__";
pub const PAYLOAD: &str = "__HOOK0_PAYLOAD__";
pub const LABEL_KEY: &str = "__HOOK0_LABEL_KEY__";
pub const LABEL_VALUE: &str = "__HOOK0_LABEL_VALUE__";

/// What a sending example carries outside its label region, every one of them.
///
/// A snippet missing one of these renders into code that does not carry the reader's own value —
/// an event sent to nowhere, or sent without the token that authorises it.
pub const REQUIRED_WHEN_SENDING: [&str; 5] = [API_URL, APPLICATION_ID, TOKEN, EVENT_TYPE, PAYLOAD];

/// What a label region carries, and what nothing outside one may.
pub const REQUIRED_IN_A_LABEL: [&str; 2] = [LABEL_KEY, LABEL_VALUE];

/// Where a verifying example gets its secret, in every language.
///
/// The screen has none to give: outside the onboarding it loads no subscription, an application may
/// have several, and printing a second secret beside the token would answer worse than not
/// answering. So the snippet reads this variable itself and the screen links to the subscription
/// that holds the value. A snippet taking the secret as a parameter instead compiles, lints and
/// renders perfectly while telling the reader nothing about where theirs comes from.
///
/// Reading it is allowed to fail, and has to. Verification asks nothing of the key it is handed:
/// `verify_webhook_signature` hashes the delivery against whatever it was given and answers a
/// mismatch, so a snippet that quietly reads an empty secret when nobody exported the variable
/// refuses every genuine delivery as a forged one — and the reader, told the signature is bad,
/// goes looking at their own signing code.
pub const SUBSCRIPTION_SECRET_VARIABLE: &str = "HOOK0_SUBSCRIPTION_SECRET";

/// Everything the dashboard substitutes. A marker outside this reaches the reader as it is written,
/// so one is refused where it is found rather than rendered.
///
/// Seven, and not one more. There is none for `occurred_at`: no example sends one, and a marker
/// tolerated that nobody uses is one that ends up half-substituted somewhere.
pub const SUBSTITUTED: [&str; 7] = [
    API_URL,
    APPLICATION_ID,
    TOKEN,
    EVENT_TYPE,
    PAYLOAD,
    LABEL_KEY,
    LABEL_VALUE,
];

/// What every substituted marker is spelled with.
const SUBSTITUTED_PREFIX: &str = "__HOOK0_";

/// One SDK, as the dashboard needs it.
#[derive(Debug, Clone, PartialEq)]
pub struct Sdk {
    /// The name the generator's registry gives the target, which is also the language the snippet
    /// is coloured as and the fragment the URL carries.
    pub target: String,
    pub display_name: String,
    pub package_name: String,
    pub registry: String,
    pub version: String,
    /// What installs it, or how it is built from a checkout when no registry carries it, and then
    /// whatever else its manifest says a reader wires before the snippet below builds.
    pub install: String,
    pub published_to_registry: bool,
    /// How far the job carrying this client goes towards proving its examples. Carried here and
    /// emitted nowhere: the artefact's type has no field for it and the screen shows no level, so
    /// what it settles is settled for whoever opens the manifest. Putting a level in front of a
    /// reader is a decision about what the panel says, and taking it is not this crate's to take.
    pub proof: manifest::Proof,
    /// The command and the job that level rests on, which reaches the artefact no further than the
    /// level does. It is what answers the obvious question a level on its own invites: a label
    /// nobody can go and check is a label worth nothing.
    pub proves: String,
    pub send: region::Labelled,
    pub label_separator: String,
    pub verify: region::Snippet,
    pub string: StringLiteral,
    /// What orders the languages on screen. Carried here and emitted nowhere: the artefact's type
    /// has no room for it, and the order of the list is what the screen reads instead.
    pub usage_share: f64,
    /// The survey that share was read off, which every one of them has to agree on.
    pub usage_source: String,
}

/// One target of the registry that the dashboard shows, with what its files are named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    pub target: String,
    /// The client's own directory, relative to the repository.
    pub client: String,
    /// What a file of this language is named with, without the dot.
    pub extension: String,
    /// What a line comment of it opens with.
    pub comment: &'static str,
}

impl Shown {
    /// Where this target's examples live.
    pub fn directory(&self) -> String {
        format!("{}/{EXAMPLES}", self.client)
    }

    fn file(&self, stem: &str) -> String {
        format!("{}/{stem}.{}", self.directory(), self.extension)
    }

    pub fn send(&self) -> String {
        self.file(SEND)
    }

    pub fn verify(&self) -> String {
        self.file(VERIFY)
    }

    pub fn manifest(&self) -> String {
        format!("{}/{}", self.directory(), manifest::FILE)
    }
}

/// Every SDK the dashboard shows, which is every target held to the whole conformance corpus.
///
/// The contract is read off the registry rather than off a coincidence of structure, for the reason
/// the registry states: reading it off an SDK owning a whole tree once held the MCP client to none
/// of the corpus while it quietly dropped two headers. So this is a property, not a list of names,
/// and nothing here has to change for the next one.
pub fn shown() -> Result<Vec<Shown>, Error> {
    let found: Vec<Shown> = targets()
        .iter()
        .filter(|target| target.contract == Contract::Whole)
        .map(|target| Shown {
            target: target.name.to_owned(),
            client: client_of(target.root),
            extension: target.language.extension.to_owned(),
            comment: target.language.comment.marker(),
        })
        .collect();

    match found.len() > limits::MAX_SDKS {
        true => Err(Error::TooManySdks {
            count: found.len(),
            ceiling: limits::MAX_SDKS,
        }),
        false => Ok(found),
    }
}

/// The client a target's emission root belongs to.
///
/// A target lands inside its client — `clients/go/generated`, `clients/php/src` — so the client is
/// the two components above whatever the target owns. Reading it off the registry rather than
/// spelling `clients/<name>` keeps the two from drifting the day a client moves.
fn client_of(root: &str) -> String {
    root.split('/').take(2).collect::<Vec<&str>>().join("/")
}

/// Every SDK the dashboard shows, read out of the tree, in the order it offers them.
pub fn sdks(tree: &Path) -> Result<Vec<Sdk>, Error> {
    let shown = shown()?;
    refuse_unclaimed(&shown, tree)?;

    let inventory = inventory(tree)?;
    let mut found = Vec::with_capacity(shown.len());
    for target in &shown {
        found.push(sdk(target, &inventory, tree)?);
    }
    refuse_mixed_vintage(&found)?;

    // Most used first, so that the languages a reader is likeliest to want are the ones they reach
    // without looking. Ties are broken by name so that the order is the same on every machine.
    found.sort_by(|left, right| {
        right
            .usage_share
            .total_cmp(&left.usage_share)
            .then_with(|| left.target.cmp(&right.target))
    });
    Ok(found)
}

/// Every share is read off the same survey, or the order is part one survey and part another.
///
/// A survey replaces the last in one go. Left unchecked, the day the figures move somebody updates
/// most of the manifests and forgets the rest, and the languages come out ordered by two different
/// years with nothing to say so — which is the whole objection to declaring the figure once per
/// language, and this is what answers it.
fn refuse_mixed_vintage(sdks: &[Sdk]) -> Result<(), Error> {
    let Some(first) = sdks.first() else {
        return Ok(());
    };
    for other in sdks {
        if other.usage_source != first.usage_source {
            return Err(Error::MixedVintage {
                first: first.target.clone(),
                first_source: first.usage_source.clone(),
                second: other.target.clone(),
                second_source: other.usage_source.clone(),
            });
        }
    }
    Ok(())
}

/// The mirror-image omission: dashboard examples sitting where no shown target is.
///
/// Nothing would ever compile them and nothing would show them, so they would rot unread — which is
/// what happens the day a target's contract narrows and its examples stay behind. Any one of the
/// three files a target owes is enough to say so. The manifest is the one a person edits, and so
/// the one most likely to be removed on purpose; two snippets left without it are the same defect
/// as three files, and reading only the manifest would call that half a tidy tree.
fn refuse_unclaimed(shown: &[Shown], tree: &Path) -> Result<(), Error> {
    let claimed: Vec<&str> = shown.iter().map(|target| target.client.as_str()).collect();

    for (name, kind) in contents(&tree.join(CLIENTS))? {
        if !kind.is_dir() {
            continue;
        }
        let client = format!("{CLIENTS}/{name}");
        if claimed.contains(&client.as_str()) {
            continue;
        }

        let directory = format!("{client}/{EXAMPLES}");
        let examples = tree.join(&directory);
        if !examples.is_dir() {
            continue;
        }
        for (left, kind) in contents(&examples)? {
            if kind.is_dir() || !owed_to_a_target(&left) {
                continue;
            }
            return Err(Error::ExamplesWithoutTarget {
                path: format!("{directory}/{left}"),
                directory: client,
            });
        }
    }
    Ok(())
}

/// Whether a file is one of the three a shown target owes.
///
/// The two snippets are recognised by their stem and whatever extension follows it, so a language
/// nothing here has ever heard of is swept the day its directory is left behind — which is the
/// same reason this sweep exists rather than a list of the languages that have one.
fn owed_to_a_target(name: &str) -> bool {
    name == manifest::FILE
        || [SEND, VERIFY].iter().any(|stem| {
            name.strip_prefix(stem)
                .is_some_and(|extension| extension.starts_with('.'))
        })
}

/// What a directory holds, by name, in the same order on every machine.
///
/// Refused rather than passed over when it cannot be read. A sweep that answers "nothing here"
/// because it could not look is a sweep that has stopped being one, and the direction it is half of
/// would go on reporting success over a tree nobody read.
fn contents(at: &Path) -> Result<Vec<(String, std::fs::FileType)>, Error> {
    let unreadable = |cause: std::io::Error| Error::ReadFile {
        path: at.display().to_string(),
        cause,
    };

    let mut found = Vec::new();
    for entry in std::fs::read_dir(at).map_err(unreadable)? {
        let entry = entry.map_err(unreadable)?;
        let kind = entry.file_type().map_err(unreadable)?;
        found.push((entry.file_name().to_string_lossy().into_owned(), kind));
    }
    found.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(found)
}

/// What the release inventory says about the packages the dashboard installs.
pub struct Inventory {
    packages: Vec<release_packages::Package>,
    mirrors: Vec<release_packages::Mirror>,
    /// The one version the SDK release is at, which is what a package versioned by the tag it is
    /// fetched under reads as.
    train: String,
    /// The packages a release flow records as published by no job, which is what says an SDK is
    /// installed from a checkout rather than from its registry.
    ///
    /// Exact rather than approximate: `check_publishers` has already refused a package that reaches
    /// neither a publish job nor this record, and refused one that reaches both. So being recorded
    /// here and being published are the two halves of one question, and reading either answers it.
    unpublished: Vec<String>,
}

/// The inventory, with the two things it has to agree on before anything is read off it: every
/// target resolves to a package, and every package reaches a registry or says why it does not.
pub fn inventory(tree: &Path) -> Result<Inventory, Error> {
    let unreadable = |cause: release_packages::Error| Error::Inventory {
        reason: cause.to_string(),
    };

    let registry: Vec<TargetRoot> = targets()
        .iter()
        .map(|target| TargetRoot {
            name: target.name.to_owned(),
            root: target.root.to_owned(),
        })
        .collect();

    let packages = release_packages::discover(&registry, tree).map_err(unreadable)?;
    release_packages::check_publishers(&packages, tree).map_err(unreadable)?;
    let train = release_packages::current_version(&release_packages::sdk_train(&packages))
        .map_err(unreadable)?
        .to_string();
    let mirrors = release_packages::mirrors(&packages).map_err(unreadable)?;

    Ok(Inventory {
        packages,
        mirrors,
        train,
        unpublished: unpublished(tree)?,
    })
}

/// Every package directory recorded as published by no job, with the reason it has none.
fn unpublished(tree: &Path) -> Result<Vec<String>, Error> {
    let path = Path::new(release_packages::NO_PUBLISH_JOB);
    if !tree.join(path).is_file() {
        return Ok(Vec::new());
    }

    let body = manifest::read_bounded(&tree.join(path), "record", limits::MAX_MANIFEST_BYTES)?;
    let document: toml::Value =
        toml::from_str(&body).map_err(|cause| Error::UnreadableManifest {
            path: release_packages::NO_PUBLISH_JOB.to_owned(),
            reason: cause.to_string(),
        })?;

    Ok(document
        .get("package")
        .and_then(toml::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|entry| entry.get("directory").and_then(toml::Value::as_str))
        .map(str::to_owned)
        .collect())
}

/// One SDK, read.
pub fn sdk(target: &Shown, inventory: &Inventory, tree: &Path) -> Result<Sdk, Error> {
    let package = inventory
        .packages
        .iter()
        .find(|package| package.target == target.target)
        .ok_or_else(|| Error::Inventory {
            reason: format!("target `{}` resolves to no package", target.target),
        })?;

    let declared = manifest::read(&existing(target, &target.manifest(), tree)?)?;
    let sending = read_text(target, &target.send(), tree)?;
    let verifying = read_text(target, &target.verify(), tree)?;

    for marker in [region::LABEL_BEGIN, region::LABEL_END] {
        if verifying.contains(marker) {
            return Err(Error::LabelMarkerInVerify {
                path: target.verify(),
                marker,
            });
        }
    }

    let send = region::labelled(
        &target.send(),
        &sending,
        target.comment,
        &declared.label_separator,
    )?;
    let verify = region::snippet(&target.verify(), &verifying, target.comment)?;

    check_markers(&target.send(), &send)?;
    only_substituted(&target.verify(), &verify.body)?;
    check_verify_secret(&target.verify(), &verify)?;

    let version = match &package.version {
        Version::Declared(site) => site.value.to_string(),
        // Nothing in the tree declares one: the host answers this package at the version of the tag
        // it is reached under, and that tag is the release every SDK goes out on.
        Version::FromTag => inventory.train.clone(),
    };
    let published = !inventory.unpublished.contains(&package.directory);

    // What installs the package is derived from the registry serving it and knows nothing about
    // this client; what else the snippet needs is the client's own to state. They are joined into
    // one block because a reader works down the screen, and finding out at the second block that
    // the first was incomplete is the failure this answers.
    let install = install::command(package, &version, published, &inventory.mirrors)?;
    let install = match &declared.snippet_also_needs {
        Some(also_needed) => format!("{install}\n{also_needed}"),
        None => install,
    };

    Ok(Sdk {
        target: target.target.clone(),
        display_name: declared.display_name,
        package_name: package.name.clone(),
        registry: install::name(package.registry).to_owned(),
        install,
        version,
        published_to_registry: published,
        proof: declared.proof,
        proves: declared.proves,
        send,
        label_separator: declared.label_separator,
        verify,
        string: declared.string,
        usage_share: declared.usage_share,
        usage_source: declared.usage_source,
    })
}

/// The path of one of a target's three files, refused by name when it is not there.
fn existing(target: &Shown, path: &str, tree: &Path) -> Result<PathBuf, Error> {
    let full = tree.join(path);
    match full.is_file() {
        true => Ok(full),
        false => Err(Error::MissingFile {
            target: target.target.clone(),
            path: path.to_owned(),
            extension: target.extension.clone(),
        }),
    }
}

fn read_text(target: &Shown, path: &str, tree: &Path) -> Result<String, Error> {
    manifest::read_bounded(
        &existing(target, path, tree)?,
        "example",
        limits::MAX_EXAMPLE_BYTES,
    )
}

/// What a sending snippet has to carry, and where.
///
/// The five values a reader types are substituted into the snippet itself; the two a label is made
/// of are substituted into the region that is repeated, and belong nowhere else — one outside it
/// would be rendered once whatever the form holds.
fn check_markers(path: &str, send: &region::Labelled) -> Result<(), Error> {
    let outside = send.body.replace(&send.label, "");

    for marker in REQUIRED_WHEN_SENDING {
        if !outside.contains(marker) {
            return Err(Error::MissingMarker {
                path: path.to_owned(),
                marker,
            });
        }
    }
    for marker in REQUIRED_IN_A_LABEL {
        if !send.label.contains(marker) {
            return Err(Error::MissingMarker {
                path: path.to_owned(),
                marker,
            });
        }
        if outside.contains(marker) {
            return Err(Error::LabelMarkerOutsideRegion {
                path: path.to_owned(),
                marker,
            });
        }
    }
    only_substituted(path, &send.body)
}

/// A verifying snippet says where its secret comes from, and reads it loudly.
///
/// See `SUBSCRIPTION_SECRET_VARIABLE`. Two refusals about the same reader. A snippet that never
/// names the variable takes its secret from nowhere and answers nothing. A snippet that names it on
/// a line also carrying an empty string literal reads no secret at all when nobody exported the
/// variable, and hands verification a key that refuses every genuine delivery.
///
/// Refused here rather than caught by a test, because neither is something to emit and then notice.
///
/// The second refusal holds one property and only that one: the line naming the variable carries no
/// empty string literal. An empty string spelled some other way is a shape this does not read.
pub fn check_verify_secret(path: &str, verify: &region::Snippet) -> Result<(), Error> {
    let mut named = false;
    for line in verify.body.lines() {
        if !line.contains(SUBSCRIPTION_SECRET_VARIABLE) {
            continue;
        }
        named = true;
        if empty_literal(line) {
            return Err(Error::SecretDefaultsToEmpty {
                path: path.to_owned(),
                variable: SUBSCRIPTION_SECRET_VARIABLE,
                line: line.trim().chars().take(MAX_QUOTED_LINE_CHARS).collect(),
            });
        }
    }

    match named {
        true => Ok(()),
        false => Err(Error::SecretOriginUnsaid {
            path: path.to_owned(),
            variable: SUBSCRIPTION_SECRET_VARIABLE,
        }),
    }
}

/// What a language opens and closes an empty string literal with.
///
/// Three characters rather than one spelling per language. A quote written twice over is an empty
/// literal wherever literals are quoted at all, so a new client is held to this the day it lands
/// and nothing here learns that a language exists.
const QUOTES: [char; 3] = ['"', '\'', '`'];

/// Whether a line writes an empty string literal: a quote character written exactly twice, with
/// nothing between the two and none of itself on either side.
///
/// Exactly twice. Three in a row open a block string rather than close an empty one, and a line
/// documenting the variable inside one of those is not the shape being looked for.
fn empty_literal(line: &str) -> bool {
    let mut characters = line.chars().peekable();
    while let Some(character) = characters.next() {
        if !QUOTES.contains(&character) {
            continue;
        }
        let mut written = 1;
        while characters.next_if_eq(&character).is_some() {
            written += 1;
        }
        if written == 2 {
            return true;
        }
    }
    false
}

/// Every marker written in a region is one the dashboard replaces.
///
/// A marker nothing replaces is not a defect the dashboard can survive: it is shown to the reader
/// exactly as it is written, in code they were told to copy.
pub fn only_substituted(path: &str, body: &str) -> Result<(), Error> {
    for written in written_markers(body) {
        if !SUBSTITUTED.contains(&written.as_str()) {
            return Err(Error::UnknownMarker {
                path: path.to_owned(),
                marker: written,
            });
        }
    }
    Ok(())
}

/// Every substitution marker a text is written with, whether or not the dashboard knows it.
///
/// A marker is the prefix followed by capitals, digits and underscores, and nothing else. The
/// narrowness is the point: the examples describe themselves in prose as being full of
/// ``the `__HOOK0_*__` words``, and a reader of this that accepted any characters between the
/// underscores would refuse the very file it was written against.
///
/// One that never closes is a marker all the same. Passing it over would let `__HOOK0_PAYLOAD` —
/// the closing underscores dropped by whoever typed it — reach the reader as itself, which is the
/// one thing reading this is for.
pub(crate) fn written_markers(body: &str) -> Vec<String> {
    body.match_indices(SUBSTITUTED_PREFIX)
        .filter_map(|(at, _)| {
            let named: String = body[at + SUBSTITUTED_PREFIX.len()..]
                .chars()
                .take_while(|character| {
                    character.is_ascii_uppercase()
                        || character.is_ascii_digit()
                        || *character == '_'
                })
                .take(MAX_QUOTED_CHARS)
                .collect();
            match named.is_empty() {
                true => None,
                false => Some(format!("{SUBSTITUTED_PREFIX}{named}")),
            }
        })
        .collect()
}

/// How much of a fragment a message quotes before it stops.
const MAX_QUOTED_CHARS: usize = 48;

/// How much of a line a message quotes before it stops.
///
/// The margin every one of these languages folds at, so a line their own linters already accept is
/// quoted whole. A shorter bound cut the quotation off before the empty literal it was about, and
/// sent the reader looking for something the message had not shown them.
const MAX_QUOTED_LINE_CHARS: usize = 120;

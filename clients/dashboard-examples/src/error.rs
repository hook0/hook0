//! Everything that stops this generator, each naming the file it was reading and what it wanted.
//!
//! A message here is read by whoever added an SDK and got a red build out of it, so each one says
//! what is missing and where it goes rather than only that something was wrong.

/// What one refusal is about.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path}: cannot be read: {cause}")]
    ReadFile {
        path: String,
        #[source]
        cause: std::io::Error,
    },

    #[error(
        "{path}: is {size} bytes, above the {maximum} one {what} is read under; raise the ceiling \
         in `limits.rs` deliberately or shorten the file"
    )]
    FileTooLarge {
        path: String,
        what: &'static str,
        size: u64,
        maximum: u64,
    },

    #[error(
        "the registry carries {count} targets held to the whole conformance corpus, above the \
         {ceiling} walked in one run"
    )]
    TooManySdks { count: usize, ceiling: usize },

    #[error(
        "target `{target}` is held to the whole conformance corpus but `{path}` is missing; every \
         such target owes the dashboard `dashboard_send.{extension}`, \
         `dashboard_verify.{extension}` and `dashboard.toml`"
    )]
    MissingFile {
        target: String,
        path: String,
        extension: String,
    },

    #[error(
        "{path}: `{directory}` holds dashboard examples, and no target of the registry is held to \
         the whole conformance corpus there; either the examples name a directory that is not a \
         client, or a target lost its contract while its examples stayed"
    )]
    ExamplesWithoutTarget { path: String, directory: String },

    #[error("{path}: is not readable as TOML: {reason}")]
    UnreadableManifest { path: String, reason: String },

    #[error("{path}: declares no `{field}`, which the dashboard cannot derive from anything else")]
    MissingField { path: String, field: &'static str },

    #[error(
        "{path}: declares `proof = \"{declared}\"`, which is none of {accepted}; the claim a \
         report prints must not overstate what the command behind it does"
    )]
    UnknownProof {
        path: String,
        declared: String,
        accepted: String,
    },

    #[error(
        "{path}: declares neither `{named}` nor `{swept}`, so nothing says what puts these \
         examples under the job that proves them; a level whose configuration can be deleted \
         without anything going red is a claim nobody can check"
    )]
    ExamplesReachUnsaid {
        path: String,
        named: &'static str,
        swept: &'static str,
    },

    #[error(
        "{path}: declares both `{named}` and `{swept}`; either a line names the directory or a \
         command reads a tree it is in, and declaring both says nothing about which one the level \
         above rests on"
    )]
    ExamplesReachSaidTwice {
        path: String,
        named: &'static str,
        swept: &'static str,
    },

    #[error(
        "{path}: `{field}` carries `{marker}`; the install block is substituted like everything \
         else on the screen, so a marker the dashboard knows renders the reader's own payload into \
         a command they were told to run, and one it does not know reaches them exactly as it is \
         written. What a reader installs or wires is the same whatever they typed into the form"
    )]
    MarkerInAlsoNeeds {
        path: String,
        field: &'static str,
        marker: String,
    },

    #[error(
        "{path}: declares a usage share with nothing above it saying where it came from; a figure \
         read off one survey is only a figure while the survey is named beside it"
    )]
    UsageShareWithoutSource { path: String },

    #[error(
        "`{first}` and `{second}` read their usage share off different surveys, so the order the \
         languages are offered in is part one and part the other.\n\n  {first}:\n{first_source}\
         \n\n  {second}:\n{second_source}\n\nOne survey replaces the last in one go: every \
         manifest moves, or none does."
    )]
    MixedVintage {
        first: String,
        first_source: String,
        second: String,
        second_source: String,
    },

    #[error(
        "{path}: `{field}` is {length} {unit} long, above the {ceiling} accepted; raise the \
         ceiling in `limits.rs` deliberately or shorten the value"
    )]
    FieldTooLong {
        path: String,
        field: &'static str,
        unit: &'static str,
        length: usize,
        ceiling: usize,
    },

    #[error(
        "{path}: `usage_share` is written as a whole number; TOML reads that as an integer and a \
         share is a fraction, so write it with a decimal point — `{value}.0`"
    )]
    UsageShareNotAFloat { path: String, value: i64 },

    #[error(
        "{path}: `usage_share` is {value}, outside the {minimum}..={maximum} a share of one \
         survey's respondents can be"
    )]
    UsageShareOutOfRange {
        path: String,
        value: f64,
        minimum: f64,
        maximum: f64,
    },

    #[error(
        "{path}: `escape` entry {at} is {found} values long; each is the pair of what is replaced \
         and what replaces it"
    )]
    EscapeNotAPair {
        path: String,
        at: usize,
        found: usize,
    },

    #[error(
        "{path}: `escape` entry {at} replaces nothing, so it would loop over the whole string \
         without consuming any of it"
    )]
    EscapeReplacesNothing { path: String, at: usize },

    #[error(
        "{path}: `escape` introduces a backslash but does not escape one first, so the rules that \
         follow escape what the earlier ones had just introduced; put the `['\\\\', …]` pair first"
    )]
    BackslashNotEscapedFirst { path: String },

    #[error(
        "{path}: `{marker}` appears {found} times; a region is delimited by exactly one of each of \
         its markers, since two openings name no single region"
    )]
    MarkerNotOnce {
        path: String,
        marker: &'static str,
        found: usize,
    },

    #[error("{path}: `{end}` comes before `{begin}`, so the region it would delimit is empty")]
    MarkersOutOfOrder {
        path: String,
        begin: &'static str,
        end: &'static str,
    },

    #[error(
        "{path}: `{marker}` is not introduced by `{comment}`, the line comment of this language; a \
         marker outside a comment would be read as code by the toolchain that proves this file"
    )]
    MarkerNotInAComment {
        path: String,
        marker: &'static str,
        comment: &'static str,
    },

    #[error(
        "{path}: `{marker}` is followed on its own line by `{trailing}`; whatever shares a line \
         with a marker is dropped along with it, so nothing may"
    )]
    CodeBesideMarker {
        path: String,
        marker: &'static str,
        trailing: String,
    },

    #[error("{path}: the region delimited by `{marker}` holds nothing")]
    EmptyRegion { path: String, marker: &'static str },

    #[error(
        "{path}: the region delimited by `{marker}` is {size} bytes, above the {ceiling} accepted"
    )]
    RegionTooLarge {
        path: String,
        marker: &'static str,
        size: usize,
        ceiling: usize,
    },

    #[error(
        "{path}: joins its labels with {separator:?}, which carries no line break, while the region \
         it joins sits on {indent} of indentation — so every repetition after the first drags that \
         whitespace into the middle of the line. Either declare a separator carrying a line break, \
         or write the region without the whitespace it sits on"
    )]
    IndentedRegionJoinedInline {
        path: String,
        separator: String,
        indent: usize,
    },

    #[error(
        "{path}: the label region is not inside the snippet region, so what the dashboard repeats \
         is not part of what it shows"
    )]
    LabelOutsideSnippet { path: String },

    #[error(
        "{path}: carries `{marker}`, and a verification example has no labels to repeat; the \
         markers belong to `dashboard_send` alone"
    )]
    LabelMarkerInVerify { path: String, marker: &'static str },

    #[error(
        "{path}: the extracted region still carries `{marker}`; a marker left in is shown to a \
         reader as if it were code"
    )]
    MarkerSurvivesExtraction { path: String, marker: String },

    #[error(
        "{path}: the snippet carries the label region {found} times; the dashboard replaces the \
         one occurrence with the labels the form holds, and cannot choose between two"
    )]
    LabelNotOnceInSnippet { path: String, found: usize },

    #[error(
        "{path}: the snippet does not carry `{marker}`, which the dashboard substitutes the form's \
         value into; without it the reader copies code that does not carry their own"
    )]
    MissingMarker { path: String, marker: &'static str },

    #[error(
        "{path}: carries `{marker}`, which is not one the dashboard substitutes; a marker nothing \
         replaces reaches the reader as it is written"
    )]
    UnknownMarker { path: String, marker: String },

    #[error(
        "{path}: carries `{marker}` outside the label region, where the dashboard substitutes \
         nothing; it belongs between the label markers"
    )]
    LabelMarkerOutsideRegion { path: String, marker: &'static str },

    #[error(
        "{path}: verifies with a secret it never says the origin of; the shown snippet has to read \
         `{variable}` itself, since the screen prints no secret and an argument arriving from \
         nowhere answers the reader nothing"
    )]
    SecretOriginUnsaid {
        path: String,
        variable: &'static str,
    },

    #[error(
        "{path}: names `{variable}` on a line that also writes an empty string literal — \
         `{line}`. That is the shape that verifies against nothing when nobody exported the \
         variable: every genuine delivery hashes to a code that does not match, so each one is \
         refused as forged while nothing says the secret was never read. Read it in whatever this \
         language raises with instead, naming the variable.\n\nOne property is held here and only \
         that one: the line naming `{variable}` carries no empty string literal. An empty string \
         spelled some other way is not a shape this reads."
    )]
    SecretDefaultsToEmpty {
        path: String,
        variable: &'static str,
        line: String,
    },

    #[error(
        "no target of the registry resolves to a package, so nothing says what installs an SDK: \
         {reason}"
    )]
    Inventory { reason: String },

    #[error(
        "`{directory}` is published to {registry} by no job and no clone recipe is written for \
         that registry; add one beside the others in `install.rs`, as `{record}` records the \
         reason the registry has no job"
    )]
    NoCloneRecipe {
        directory: String,
        registry: &'static str,
        record: &'static str,
    },

    #[error(
        "`{name}` is published to Maven Central, whose coordinates are `<group>:<artifact>`, and \
         this one carries no colon"
    )]
    NotAMavenCoordinate { name: String },

    #[error("`{directory}` is fetched by URL and this release pushes it to no mirror: {reason}")]
    NoMirror { directory: String, reason: String },
}

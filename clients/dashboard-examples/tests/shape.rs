//! What a region comes out as, and what it is refused for.
//!
//! Written against text rather than against files, because the shapes that matter are the ones a
//! formatter produces: the marker moved up to the end of a line of code, the trailing separator put
//! back, the indentation the region sits on. Each of those is a string, and a string is what this
//! hands the reader.

use hook0_dashboard_examples::{
    LABEL_KEY, LABEL_VALUE, REQUIRED_IN_A_LABEL, REQUIRED_WHEN_SENDING,
    SUBSCRIPTION_SECRET_VARIABLE, check_verify_secret, error::Error, install, inventory,
    only_substituted, region, sdk, sdks, shown,
};
use release_packages::{Kind, Package, Registry, Train, Version};

mod common;

/// The comment a language of this shape opens with.
const SLASHES: &str = "//";

/// What joins two labels, and what a formatter therefore puts at the end of one.
const COMMA: &str = ",\n";

/// A snippet whose closing label marker a formatter moved up to the end of the line of code above
/// it, which is what `rustfmt` and `gofmt` both do and what neither can be talked out of.
const MOVED_UP: &str = r#"// hook0:snippet:begin
send(Event {
    labels: vec![
        // hook0:label:begin
        ("__HOOK0_LABEL_KEY__", "__HOOK0_LABEL_VALUE__"), // hook0:label:end
    ],
});
// hook0:snippet:end
"#;

/// The same snippet with both markers on lines of their own, which is how it is written before a
/// formatter has been near it.
const ON_ITS_OWN_LINE: &str = r#"// hook0:snippet:begin
send(Event {
    labels: vec![
        // hook0:label:begin
        ("__HOOK0_LABEL_KEY__", "__HOOK0_LABEL_VALUE__"),
        // hook0:label:end
    ],
});
// hook0:snippet:end
"#;

/// A moved marker and one left where it was written come out as the same region.
///
/// This is the whole reason a marker is read as a token rather than as a line. Reading lines would
/// take the line of code the formatter moved the marker onto, and the snippet would lose the label
/// it is there to show.
#[test]
fn a_formatter_moving_a_marker_changes_nothing() {
    let moved = region::labelled("moved", MOVED_UP, SLASHES, COMMA).expect("the region is unread");
    let written =
        region::labelled("written", ON_ITS_OWN_LINE, SLASHES, COMMA).expect("the region is unread");

    assert_eq!(moved, written);
    assert_eq!(
        moved.label,
        r#"        ("__HOOK0_LABEL_KEY__", "__HOOK0_LABEL_VALUE__")"#
    );
}

/// The region keeps the whitespace it sits on, and drops the separator a formatter put after it.
///
/// Both halves matter to the same operation: every repetition brings its own indentation, and the
/// separator that joins them is written once, between them.
#[test]
fn a_region_keeps_its_indentation_and_drops_its_separator() {
    let read = region::labelled("moved", MOVED_UP, SLASHES, COMMA).expect("the region is unread");

    assert!(
        read.label.starts_with("        ("),
        "the indentation is gone"
    );
    assert!(!read.label.ends_with(','), "the separator was kept");
    assert!(
        read.body.contains(&read.label),
        "the snippet does not carry the label it is repeated from"
    );
}

/// Repeating the label is a string replacement, and repeating it no times leaves a container the
/// language accepts.
#[test]
fn the_label_repeats_by_replacement() {
    let read = region::labelled("moved", MOVED_UP, SLASHES, COMMA).expect("the region is unread");

    let two = read.body.replace(
        &read.label,
        &[read.label.as_str(), read.label.as_str()].join(COMMA),
    );
    assert_eq!(two.matches(LABEL_KEY).count(), 2);
    assert_eq!(two.matches(LABEL_VALUE).count(), 2);

    let none = read.body.replace(&read.label, "");
    assert!(!none.contains(LABEL_KEY), "a label survived being removed");
    assert!(
        none.contains("labels: vec![") && none.contains("],"),
        "removing every label did not leave the container behind: {none}"
    );
}

/// No marker of this crate survives into what a reader is shown.
#[test]
fn no_marker_survives_a_region() {
    let read = region::labelled("moved", MOVED_UP, SLASHES, COMMA).expect("the region is unread");

    for left in ["hook0:snippet", "hook0:label", SLASHES] {
        assert!(
            !read.label.contains(left),
            "the label still carries `{left}`"
        );
    }
    assert!(
        !read.body.contains("hook0:label"),
        "the snippet still carries a label marker"
    );
}

/// A path spelled with two colons is not a marker, and a misspelled marker is.
///
/// The prefix is read rather than the four names, so that `hook0:lable:begin` is caught rather than
/// silently shown; a word has to follow the prefix, so that `hook0::Client` is not.
#[test]
fn a_path_is_not_a_marker_and_a_typo_is() {
    let path = "// hook0:snippet:begin\nlet client = hook0::Client::new();\n// hook0:snippet:end\n";
    assert_eq!(
        region::snippet("path", path, SLASHES)
            .expect("a path was read as a marker")
            .body,
        "let client = hook0::Client::new();"
    );

    let typo = "// hook0:snippet:begin\nsend(); // hook0:lable:begin\n// hook0:snippet:end\n";
    assert!(
        matches!(
            region::snippet("typo", typo, SLASHES),
            Err(Error::MarkerSurvivesExtraction { .. })
        ),
        "a misspelled marker was shown to the reader as code"
    );
}

/// A marker written outside a comment is refused.
///
/// It would not be a marker at all to the toolchain that proves the file: it would be code, and the
/// file would not compile — but it would be read here, which is worse than either.
#[test]
fn a_marker_has_to_be_a_comment() {
    let bare = "hook0:snippet:begin\nsend();\n// hook0:snippet:end\n";
    assert!(matches!(
        region::snippet("bare", bare, SLASHES),
        Err(Error::MarkerNotInAComment { .. })
    ));
}

/// Anything sharing a line with a marker is refused, since it would be cut away with it.
#[test]
fn nothing_shares_a_line_with_a_marker() {
    let shared = "// hook0:snippet:begin send();\nsend();\n// hook0:snippet:end\n";
    assert!(matches!(
        region::snippet("shared", shared, SLASHES),
        Err(Error::CodeBesideMarker { .. })
    ));
}

/// Two openings name no single region.
#[test]
fn a_marker_appears_once() {
    let twice =
        "// hook0:snippet:begin\nsend();\n// hook0:snippet:begin\nsend();\n// hook0:snippet:end\n";
    assert!(matches!(
        region::snippet("twice", twice, SLASHES),
        Err(Error::MarkerNotOnce { .. })
    ));

    let never = "send();\n";
    assert!(matches!(
        region::snippet("never", never, SLASHES),
        Err(Error::MarkerNotOnce { .. })
    ));
}

/// A label the snippet does not contain is a label the dashboard would repeat out of sight.
#[test]
fn the_label_lives_inside_the_snippet() {
    let outside = "// hook0:label:begin\n(\"__HOOK0_LABEL_KEY__\", \"__HOOK0_LABEL_VALUE__\")\n// \
                   hook0:label:end\n// hook0:snippet:begin\nsend();\n// hook0:snippet:end\n";
    assert!(matches!(
        region::labelled("outside", outside, SLASHES, COMMA),
        Err(Error::LabelOutsideSnippet { .. })
    ));
}

/// Every SDK the dashboard shows carries, in the right half of its snippet, every marker the
/// dashboard substitutes.
#[test]
fn every_snippet_carries_the_markers_it_is_rendered_by() {
    let tree = common::tree();
    let inventory = inventory(&tree).expect("the release inventory does not read");

    for target in shown().expect("the registry is unreadable") {
        let read = sdk(&target, &inventory, &tree)
            .unwrap_or_else(|cause| panic!("`{}` does not read: {cause}", target.target));
        let outside = read.send.body.replace(&read.send.label, "");

        for marker in REQUIRED_WHEN_SENDING {
            assert!(
                outside.contains(marker),
                "`{}` sends without carrying {marker}",
                read.target
            );
        }
        for marker in REQUIRED_IN_A_LABEL {
            assert!(
                read.send.label.contains(marker),
                "`{}` renders a label without {marker}",
                read.target
            );
            assert!(
                !outside.contains(marker),
                "`{}` carries {marker} where nothing repeats it",
                read.target
            );
        }
    }
}

/// A verifying snippet that hides where its secret comes from is refused.
///
/// The screen has none to give: outside the onboarding it loads no subscription, an application may
/// have several, and printing a second secret beside the token would answer worse than not
/// answering. So the snippet reads the variable itself and the screen links to the subscription
/// holding the value.
///
/// What makes this worth refusing rather than reviewing is that the other shape is invisible from
/// inside one language: a `verify` taking the secret as a parameter compiles, lints and renders
/// perfectly while telling the reader nothing. Most of them drifted that way, each plausible on its
/// own, and nothing went red.
///
/// The variable name is the assertion because it is the one thing every language spells the same.
#[test]
fn a_verify_that_hides_where_its_secret_comes_from_is_refused() {
    let says = region::snippet(
        "says",
        "// hook0:snippet:begin\nlet secret = env(\"HOOK0_SUBSCRIPTION_SECRET\");\n// hook0:snippet:end\n",
        SLASHES,
    )
    .expect("the region does not read");
    assert!(
        check_verify_secret("says", &says).is_ok(),
        "a snippet reading `{SUBSCRIPTION_SECRET_VARIABLE}` was refused"
    );

    let hides = region::snippet(
        "hides",
        "// hook0:snippet:begin\nfn accept(secret: &str) -> bool { verify(secret) }\n// hook0:snippet:end\n",
        SLASHES,
    )
    .expect("the region does not read");
    assert!(
        matches!(
            check_verify_secret("hides", &hides),
            Err(Error::SecretOriginUnsaid { .. })
        ),
        "a snippet taking its secret from nowhere was emitted, so a reader is left with an \
         argument they have no way to fill"
    );
}

/// A verifying snippet that reads the variable and falls back to an empty secret is refused.
///
/// Naming the variable is not enough on its own. Verification asks nothing of the key it is handed:
/// it hashes the delivery against whatever it was given and answers a mismatch, so a snippet that
/// substitutes an empty string for a variable nobody exported refuses every genuine delivery as a
/// forged one. The reader is told the signature is bad and goes looking at their own signing code,
/// which is the one place the defect is not.
#[test]
fn a_verify_that_falls_back_to_an_empty_secret_is_refused() {
    let read = |body: &str| {
        region::snippet(
            "region",
            &format!("// hook0:snippet:begin\n{body}\n// hook0:snippet:end\n"),
            SLASHES,
        )
        .expect("the region does not read")
    };

    for silent in [
        r#"let secret = env("HOOK0_SUBSCRIPTION_SECRET", "");"#,
        r#"secret = ENV.fetch("HOOK0_SUBSCRIPTION_SECRET", "")"#,
        r#"const secret = environ.get("HOOK0_SUBSCRIPTION_SECRET") orelse "";"#,
        r#"$secret = getenv('HOOK0_SUBSCRIPTION_SECRET') ?: '';"#,
    ] {
        let refused = check_verify_secret("silent", &read(silent))
            .expect_err("a snippet defaulting its secret to nothing was emitted");
        assert!(matches!(refused, Error::SecretDefaultsToEmpty { .. }));
        assert!(
            refused.to_string().contains(SUBSCRIPTION_SECRET_VARIABLE),
            "the refusal does not name the variable: {refused}"
        );
    }

    // The same read, raising instead. Every one of these is what one of these languages now writes,
    // and the guard has nothing to say about any of them.
    for loud in [
        r#"let secret = std::env::var("HOOK0_SUBSCRIPTION_SECRET").expect("HOOK0_SUBSCRIPTION_SECRET is not set");"#,
        r#"secret = os.environ["HOOK0_SUBSCRIPTION_SECRET"]"#,
        r#"secret, set := os.LookupEnv("HOOK0_SUBSCRIPTION_SECRET")"#,
        r#"const secret = environ.get("HOOK0_SUBSCRIPTION_SECRET") orelse return error.SubscriptionSecretNotSet;"#,
    ] {
        assert!(
            check_verify_secret("loud", &read(loud)).is_ok(),
            "a snippet raising on a secret nobody set was refused: {loud}"
        );
    }
}

/// An empty literal that is not on the line naming the variable is left alone.
///
/// The property is about one line, and it has to stay about one line. A verifying snippet is a
/// handler with a body, and a body writes empty strings for its own reasons — an unnamed header, a
/// response with nothing in it. Refusing those would push the next person to write the read some
/// way the guard cannot see, which is the shape this exists to stop.
#[test]
fn an_empty_literal_away_from_the_secret_is_left_alone() {
    let elsewhere = "// hook0:snippet:begin\nlet header = headers.get(\"x-hook0-signature\", \"\");\n\
                     let secret = env(\"HOOK0_SUBSCRIPTION_SECRET\");\nlet body = \"\";\n\
                     // hook0:snippet:end\n";

    let read = region::snippet("elsewhere", elsewhere, SLASHES).expect("the region does not read");
    assert!(
        check_verify_secret("elsewhere", &read).is_ok(),
        "an empty literal on a line that names no secret was read as a fallback"
    );
}

/// Every SDK installs from somewhere, and says which.
#[test]
fn every_sdk_says_what_installs_it() {
    for read in sdks(&common::tree()).expect("the dashboard examples do not read") {
        assert!(
            !read.install.is_empty(),
            "`{}` installs from nothing",
            read.target
        );
        assert!(
            !read.version.is_empty(),
            "`{}` is at no version",
            read.target
        );
        assert!(
            !read.registry.is_empty(),
            "`{}` names no registry",
            read.target
        );
    }
}

/// What installs a package is a total function of the registry serving it, and the two ecosystems
/// no release reaches today are part of that total.
///
/// Maven publishes nothing from a command line, so what a reader does there is declare a dependency
/// — and a coordinate that is not `<group>:<artifact>` is refused rather than rendered into half an
/// element. An ecosystem with no clone recipe written for it is refused too, naming the file where
/// the reason its packages reach no registry is recorded: guessing one would put a command on
/// screen that nobody has ever run.
#[test]
fn an_ecosystem_nothing_reaches_today_is_written_or_refused() {
    let package = |registry: Registry, name: &str| Package {
        target: "example".to_owned(),
        directory: "clients/example".to_owned(),
        manifest: "clients/example/pom.xml".to_owned(),
        kind: Kind::Pom,
        registry,
        name: name.to_owned(),
        version: Version::FromTag,
        train: Train::Sdk,
    };

    let declared = install::command(
        &package(Registry::MavenCentral, "com.hook0:hook0-client"),
        "2.0.2",
        true,
        &[],
    )
    .expect("Maven Central says nothing about what installs from it");
    assert!(declared.contains("<groupId>com.hook0</groupId>"));
    assert!(declared.contains("<artifactId>hook0-client</artifactId>"));
    assert!(declared.contains("<version>2.0.2</version>"));

    assert!(matches!(
        install::command(
            &package(Registry::MavenCentral, "hook0-client"),
            "2.0.2",
            true,
            &[]
        ),
        Err(Error::NotAMavenCoordinate { .. })
    ));

    assert!(matches!(
        install::command(
            &package(Registry::CratesIo, "hook0-client"),
            "2.0.2",
            false,
            &[]
        ),
        Err(Error::NoCloneRecipe { .. })
    ));
}

/// The prose an example describes itself with is not a marker.
///
/// Every one of them carries the sentence the canonical file introduced — ``the `__HOOK0_*__`
/// words are string literals`` — and a reader of markers written widely enough to match `*` between
/// the underscores refuses the file it was written against. So a marker is the prefix followed by
/// capitals, digits and underscores, and the sentence goes past untouched.
#[test]
fn the_sentence_the_examples_describe_themselves_with_is_not_a_marker() {
    let described = "// hook0:snippet:begin\n// The `__HOOK0_*__` words are string literals, which \
                     is what lets a file full of\n// them compile.\nsend(\"__HOOK0_PAYLOAD__\");\n\
                     // hook0:snippet:end\n";

    let read = region::snippet("described", described, SLASHES).expect("the region is unread");
    assert!(read.body.contains("__HOOK0_*__"), "the sentence was lost");
    assert!(
        only_substituted("described", &read.body).is_ok(),
        "the sentence the examples describe themselves with was read as a marker"
    );

    let truncated = "// hook0:snippet:begin\nsend(\"__HOOK0_PAYLOAD\");\n// hook0:snippet:end\n";
    let read = region::snippet("truncated", truncated, SLASHES).expect("the region is unread");
    assert!(
        matches!(
            only_substituted("truncated", &read.body),
            Err(Error::UnknownMarker { .. })
        ),
        "a marker whose closing underscores were dropped went through"
    );
}

/// A separator that is a line break and no punctuation takes nothing off the region.
///
/// Go is the one that needs it: a composite literal whose `}` opens a line is a
/// syntax error *without* a trailing comma, because a semicolon is inserted at the line break — the
/// exact opposite of what Java's variadic argument list allows. Its manifest therefore declares
/// `"\n"`, whose punctuation is empty, and a suffix taken off unguarded empties the whole region.
#[test]
fn a_separator_of_pure_whitespace_takes_nothing_off() {
    let go = "// hook0:snippet:begin\nlabels := map[string]string{}\n// hook0:label:begin\nlabels\
              [\"__HOOK0_LABEL_KEY__\"] = \"__HOOK0_LABEL_VALUE__\" // hook0:label:end\nsend(labels)\
              \n// hook0:snippet:end\n";

    let read = region::labelled("go", go, SLASHES, "\n").expect("the region is unread");
    assert_eq!(
        read.label, "labels[\"__HOOK0_LABEL_KEY__\"] = \"__HOOK0_LABEL_VALUE__\"",
        "a separator with no punctuation emptied the region"
    );
    assert!(read.body.contains(&read.label));
}

/// Repeated two and three times, every copy sits where the first one did.
///
/// One label proves nothing here: cutting at the marker rather than at the start of the line it is
/// written on produces code that still compiles, and doubles the indentation of the first copy
/// alone. It takes a second copy to see the two disagree.
#[test]
fn every_repetition_is_indented_like_the_first() {
    let read = region::labelled("moved", MOVED_UP, SLASHES, COMMA).expect("the region is unread");
    let indent = |line: &str| line.len() - line.trim_start().len();

    for copies in [2, 3] {
        let joined = vec![read.label.as_str(); copies].join(COMMA);
        let rendered = read.body.replace(&read.label, &joined);

        let opens: Vec<usize> = rendered
            .lines()
            .filter(|line| line.trim_start().starts_with('('))
            .map(indent)
            .collect();
        assert_eq!(
            opens.len(),
            copies,
            "{copies} copies did not produce {copies} of them"
        );
        assert!(
            opens.windows(2).all(|pair| pair[0] == pair[1]),
            "the copies are indented differently from one another: {opens:?}"
        );
    }
}

/// A region sitting on indentation and a separator with no line break in it do not go together.
///
/// The two declarations are each defensible and their combination is not: the repetitions land on
/// one line and every one after the first drags the region's indent into the middle of it. Java
/// declared `", "` for a while and three labels came out as one line with twenty-two spaces between
/// each pair — code that compiles and that nobody would copy.
#[test]
fn an_indented_region_is_not_joined_on_one_line() {
    let indented = "// hook0:snippet:begin\nMap.of(\n    // hook0:label:begin\n    \
                    \"__HOOK0_LABEL_KEY__\", \"__HOOK0_LABEL_VALUE__\" // hook0:label:end\n    );\n\
                    // hook0:snippet:end\n";

    let refused = region::labelled("indented", indented, SLASHES, ", ")
        .expect_err("a region sitting on an indent was joined onto one line");
    assert!(matches!(refused, Error::IndentedRegionJoinedInline { .. }));
    let said = refused.to_string();
    assert!(
        said.contains("line break"),
        "the refusal does not say how to settle it: {said}"
    );

    // The same region joined by a separator that starts a line is what every one of them declares,
    // and it is accepted.
    region::labelled("indented", indented, SLASHES, ",\n").expect("the region is unread");

    // So is a region written without the whitespace it would otherwise sit on, joined inline.
    let flush = "// hook0:snippet:begin\nMap.of(\n// hook0:label:begin\n\"__HOOK0_LABEL_KEY__\", \
                 \"__HOOK0_LABEL_VALUE__\" // hook0:label:end\n);\n// hook0:snippet:end\n";
    region::labelled("flush", flush, SLASHES, ", ").expect("a flush region was refused");
}

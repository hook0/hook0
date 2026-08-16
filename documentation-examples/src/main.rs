//! The checker, run over a real repository.
//!
//! The report is written for whoever opens a red pipeline: what was proven, at what level, and —
//! when something failed — which page and which line a reader would have copied it from.

use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use hook0_documentation_examples::{
    Documentation, Error, Language, Proven, SDK_REFERENCE, discover, project, registry,
};

/// The most languages one invocation may be narrowed to.
const MAX_SELECTION: usize = 16;

/// What the command line said.
struct Invocation {
    repository: PathBuf,
    /// Where projects are assembled. A temporary directory unless one was named.
    work: Option<PathBuf>,
    /// The languages to prove, empty meaning every one of them.
    only: BTreeSet<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("\ndocumentation examples: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool, Error> {
    let invocation = read_invocation()?;
    let targets = registry();
    let sdk = invocation.repository.join(SDK_REFERENCE);

    let documentation = discover(&targets, &sdk)?;
    announce(&documentation, &targets.len(), &sdk);

    let work = match &invocation.work {
        Some(named) => Work::Named(named.clone()),
        None => Work::Temporary(tempdir()?),
    };

    let mut results = Vec::new();
    for language in &documentation.languages {
        if !invocation.only.is_empty() && !invocation.only.contains(&language.name) {
            continue;
        }
        let root = project::root_for(work.path(), &language.name);
        let client = invocation.repository.join(&language.client);
        println!(
            "  {:<12} {:<13} {:>2} examples  {}",
            language.name,
            language.manifest.proof.word(),
            language.examples.len(),
            language.manifest.proves,
        );
        let proven = project::prove(language, &root, &invocation.repository, &client)?;
        report(language, &proven);
        results.push(proven);
    }

    Ok(summarise(
        &documentation,
        &results,
        !invocation.only.is_empty(),
    ))
}

/// A place to assemble in, kept alive for as long as the run needs it.
enum Work {
    Temporary(tempfile::TempDir),
    Named(PathBuf),
}

impl Work {
    fn path(&self) -> &Path {
        match self {
            Work::Temporary(directory) => directory.path(),
            Work::Named(path) => path.as_path(),
        }
    }
}

fn tempdir() -> Result<tempfile::TempDir, Error> {
    tempfile::Builder::new()
        .prefix("hook0-documentation-examples-")
        .tempdir()
        .map_err(|cause| Error::WriteFile {
            path: env::temp_dir().display().to_string(),
            cause,
        })
}

fn read_invocation() -> Result<Invocation, Error> {
    // The repository is the directory above this crate, which is where it is checked out. It is
    // resolved at build time so that the checker can be run from anywhere.
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut invocation = Invocation {
        repository,
        work: None,
        only: BTreeSet::new(),
    };

    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--only" => {
                let Some(language) = arguments.next() else {
                    return Err(usage("--only names no language"));
                };
                if invocation.only.len() == MAX_SELECTION {
                    return Err(usage("--only was given more than sixteen languages"));
                }
                invocation.only.insert(language);
            }
            "--work-dir" => {
                let Some(path) = arguments.next() else {
                    return Err(usage("--work-dir names no directory"));
                };
                invocation.work = Some(PathBuf::from(path));
            }
            "--repository" => {
                let Some(path) = arguments.next() else {
                    return Err(usage("--repository names no directory"));
                };
                invocation.repository = PathBuf::from(path);
            }
            other => return Err(usage(&format!("`{other}` means nothing here"))),
        }
    }

    Ok(invocation)
}

fn usage(detail: &str) -> Error {
    Error::ExamplesRefused {
        summary: format!(
            "{detail}\n\nusage: documentation-examples [--only <language>]... \
             [--work-dir <path>] [--repository <path>]"
        ),
    }
}

fn announce(documentation: &Documentation, targets: &usize, sdk: &Path) {
    println!(
        "\n{} pages under {}, {targets} targets in the generator's registry\n",
        documentation.pages.len(),
        sdk.display(),
    );
}

/// What one language answered, printed as it finishes rather than at the end.
fn report(language: &Language, proven: &Proven) {
    if proven.succeeded() {
        println!(
            "  {:<12} ok, {} in {}\n",
            "",
            language.manifest.proof.word(),
            took(proven.took),
        );
        return;
    }

    println!("\n  {} FAILED after {}", language.name, took(proven.took));
    for ran in proven.ran.iter().filter(|ran| !ran.succeeded()) {
        println!(
            "  $ {}\n  exit {}{}",
            ran.command,
            match ran.code {
                Some(code) => code.to_string(),
                None => "by a signal".to_owned(),
            },
            if ran.cut { ", output cut" } else { "" },
        );
        for line in ran.output.lines() {
            println!("  | {line}");
        }
    }

    let blamed = proven.blamed();
    if blamed.is_empty() {
        println!(
            "  the toolchain named no assembled example, so this is the {} project itself \
             rather than a snippet",
            language.name,
        );
    } else {
        println!(
            "  examples that did not {}:",
            language.manifest.proof.verb()
        );
        for one in blamed {
            println!("    {} -> {}", one.file, one.example.locate());
        }
    }
    println!();
}

/// The last thing printed, and the verdict.
fn summarise(documentation: &Documentation, results: &[Proven], partial: bool) -> bool {
    let examples: usize = results.iter().map(|proven| proven.assembled.len()).sum();
    let failed: Vec<&Proven> = results
        .iter()
        .filter(|proven| !proven.succeeded())
        .collect();

    println!("─────────────────────────────────────────────────────────────");
    println!(
        "{examples} examples across {} languages, {} proven, {} refused",
        results.len(),
        examples
            - failed
                .iter()
                .map(|proven| proven.blamed().len())
                .sum::<usize>(),
        failed
            .iter()
            .map(|proven| proven.blamed().len())
            .sum::<usize>(),
    );

    if !documentation.undocumented.is_empty() {
        println!(
            "targets the registry produces that no page here documents: {}",
            documentation.undocumented.join(", "),
        );
    }

    if partial {
        println!("PARTIAL RUN — --only was given, so this proves less than the pipeline does");
    }

    if failed.is_empty() {
        println!("every example is real");
        return true;
    }

    println!(
        "refused: {}",
        failed
            .iter()
            .map(|proven| proven.language.as_str())
            .collect::<Vec<&str>>()
            .join(", "),
    );
    false
}

fn took(duration: Duration) -> String {
    format!("{:.1}s", duration.as_secs_f64())
}

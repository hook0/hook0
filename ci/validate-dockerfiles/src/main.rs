use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use walkdir::WalkDir;

fn main() -> ExitCode {
    let root = std::env::var_os("VALIDATE_DOCKERFILES_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    match run(&root) {
        Ok(0) => {
            println!("\n[OK] All Dockerfile checks passed.");
            ExitCode::SUCCESS
        }
        Ok(n) => {
            eprintln!("\n[FAIL] {n} issue(s) detected. See details above.");
            ExitCode::from(1)
        }
        Err(e) => {
            eprintln!("[ERROR] {e}");
            ExitCode::from(2)
        }
    }
}

fn run(root: &Path) -> Result<u32, String> {
    let members = read_workspace_members(root)?;
    println!("Cargo workspace members ({}):", members.len());
    for m in &members {
        println!("  - {m}");
    }
    println!();

    let dockerfiles = find_dockerfiles(root);
    println!("Dockerfiles discovered: {}", dockerfiles.len());
    for df in &dockerfiles {
        println!("  - {}", df.strip_prefix(root).unwrap_or(df).display());
    }
    println!();

    let mut issues = 0u32;
    for df in &dockerfiles {
        let rel = df.strip_prefix(root).unwrap_or(df);
        let body = fs::read_to_string(df).map_err(|e| format!("read {}: {e}", df.display()))?;
        issues += report_workspace_coverage(rel, &body, &members);
        issues += check_dockerignore_consistency(root, df, rel, &body);
    }
    Ok(issues)
}

fn read_workspace_members(root: &Path) -> Result<BTreeSet<String>, String> {
    let path = root.join("Cargo.toml");
    let content = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&content).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let members = value
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .ok_or_else(|| "Cargo.toml has no [workspace].members".to_string())?;
    let mut out = BTreeSet::new();
    for v in members {
        if let Some(s) = v.as_str() {
            out.insert(s.to_string());
        }
    }
    Ok(out)
}

fn find_dockerfiles(root: &Path) -> Vec<PathBuf> {
    const SKIP_DIRS: &[&str] = &[
        "target",
        "node_modules",
        ".git",
        "build-context",
        "dist",
        ".claude",
        ".conductor",
        ".gstack",
        ".playwright-mcp",
    ];
    let walker = WalkDir::new(root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !SKIP_DIRS.iter().any(|d| name == *d)
    });
    let mut found = Vec::new();
    for entry in walker.flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.ends_with(".dockerignore") {
            continue;
        }
        if name == "Dockerfile" || name.starts_with("Dockerfile.") {
            found.push(entry.path().to_path_buf());
        }
    }
    found.sort();
    found
}

/// How a Dockerfile brings a path of the repository into its build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceMechanism {
    BindMount,
    Copy,
}

impl SourceMechanism {
    fn label(self) -> &'static str {
        match self {
            Self::BindMount => "bind mounts",
            Self::Copy => "COPY instructions",
        }
    }

    fn instruction_for(self, member: &str) -> String {
        match self {
            Self::BindMount => format!("--mount=type=bind,source={member},target={member}"),
            Self::Copy => format!("COPY {member} {member}"),
        }
    }
}

/// Which workspace members a Dockerfile makes available to `cargo`, and how.
#[derive(Debug)]
struct WorkspaceCoverage {
    mechanism: SourceMechanism,
    missing: Vec<String>,
    total: usize,
}

/// A Dockerfile builds the cargo workspace when it brings in the workspace manifest **and** at
/// least one of its members. A crate built from its own context copies its own `Cargo.toml` but no
/// member directory, so it is not concerned by workspace-wide coverage.
fn workspace_coverage(body: &str, members: &BTreeSet<String>) -> Option<WorkspaceCoverage> {
    let provided = provided_paths(body);
    if !provided.contains_key("Cargo.toml") {
        return None;
    }
    let mechanisms: Vec<SourceMechanism> = members
        .iter()
        .filter_map(|m| provided.get(m.as_str()).copied())
        .collect();
    if mechanisms.is_empty() {
        return None;
    }
    let mechanism = if mechanisms.contains(&SourceMechanism::BindMount) {
        SourceMechanism::BindMount
    } else {
        SourceMechanism::Copy
    };
    let missing = members
        .iter()
        .filter(|m| !provided.contains_key(m.as_str()))
        .cloned()
        .collect();
    Some(WorkspaceCoverage {
        mechanism,
        missing,
        total: members.len(),
    })
}

fn report_workspace_coverage(rel: &Path, body: &str, members: &BTreeSet<String>) -> u32 {
    let Some(coverage) = workspace_coverage(body, members) else {
        return 0;
    };
    if coverage.missing.is_empty() {
        println!(
            "[OK]   {} provides all {} workspace members ({})",
            rel.display(),
            coverage.total,
            coverage.mechanism.label()
        );
        return 0;
    }
    eprintln!(
        "[FAIL] {} declares cargo workspace usage but is missing {} for:",
        rel.display(),
        coverage.mechanism.label()
    );
    for m in &coverage.missing {
        eprintln!("         - {m}");
    }
    eprintln!(
        "       cargo refuses to build because the workspace Cargo.toml lists these as members."
    );
    eprintln!("       Add for each:");
    for m in &coverage.missing {
        eprintln!("         {}", coverage.mechanism.instruction_for(m));
    }
    1
}

/// Every repository path a Dockerfile brings into its build, keyed by path and mapped to the
/// mechanism used. A path provided both ways is reported as a bind mount.
fn provided_paths(body: &str) -> BTreeMap<String, SourceMechanism> {
    let mut out = BTreeMap::new();
    for src in extract_copy_sources(body) {
        out.insert(normalize_source(&src), SourceMechanism::Copy);
    }
    for src in extract_bind_sources(body) {
        out.insert(normalize_source(&src), SourceMechanism::BindMount);
    }
    out
}

fn normalize_source(raw: &str) -> String {
    raw.trim_start_matches("./")
        .trim_end_matches('/')
        .to_string()
}

fn extract_bind_sources(body: &str) -> BTreeSet<String> {
    const NEEDLE: &str = "--mount=type=bind,";
    let mut out = BTreeSet::new();
    for line in body.lines() {
        let mut s = line;
        while let Some(idx) = s.find(NEEDLE) {
            let rest = &s[idx + NEEDLE.len()..];
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '\\')
                .unwrap_or(rest.len());
            let spec = &rest[..end];
            for kv in spec.split(',') {
                if let Some(v) = kv.strip_prefix("source=") {
                    out.insert(v.to_string());
                }
            }
            s = &rest[end..];
        }
    }
    out
}

fn extract_copy_sources(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in body.lines() {
        out.extend(copy_sources_of_line(line));
    }
    out
}

fn copy_sources_of_line(line: &str) -> Vec<String> {
    let t = line.trim();
    let Some(rest) = t.strip_prefix("COPY ").or_else(|| t.strip_prefix("ADD ")) else {
        return Vec::new();
    };
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.iter().any(|p| p.starts_with("--from=")) {
        return Vec::new();
    }
    let non_flag: Vec<&str> = parts.into_iter().filter(|p| !p.starts_with("--")).collect();
    if non_flag.len() < 2 {
        return Vec::new();
    }
    non_flag[..non_flag.len() - 1]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn read_dockerignore(path: &Path) -> Vec<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect()
}

fn resolve_active_ignore(root: &Path, df_abs: &Path) -> PathBuf {
    let n = df_abs
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let per_dockerfile = df_abs.with_file_name(format!("{n}.dockerignore"));
    if per_dockerfile.exists() {
        per_dockerfile
    } else {
        root.join(".dockerignore")
    }
}

fn check_dockerignore_consistency(root: &Path, df_abs: &Path, df_rel: &Path, body: &str) -> u32 {
    let active_ignore = resolve_active_ignore(root, df_abs);
    let patterns = read_dockerignore(&active_ignore);
    if patterns.is_empty() {
        return 0;
    }
    let copies = extract_copy_sources(body);
    let mut issues = 0u32;
    for src in &copies {
        let normalized = normalize_source(src);
        if normalized.is_empty() || normalized == "." {
            continue;
        }
        let head = normalized.split('/').next().unwrap_or(&normalized);
        for pat in &patterns {
            if pat.starts_with('!') {
                continue;
            }
            let pat_norm = pat.trim_start_matches('/').trim_end_matches('/');
            if pat_norm == head || pat_norm == normalized {
                eprintln!(
                    "[FAIL] {} `COPY {}` is excluded by {} (pattern `{}`)",
                    df_rel.display(),
                    src,
                    active_ignore
                        .strip_prefix(root)
                        .unwrap_or(&active_ignore)
                        .display(),
                    pat
                );
                eprintln!(
                    "       The build context will not contain `{src}`. Provide a per-Dockerfile {}.dockerignore that does not exclude it.",
                    df_rel.display()
                );
                issues += 1;
            }
        }
    }
    if issues == 0 && !copies.is_empty() {
        println!(
            "[OK]   {} COPY sources consistent with {}",
            df_rel.display(),
            active_ignore
                .strip_prefix(root)
                .unwrap_or(&active_ignore)
                .display()
        );
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .canonicalize()
            .expect("the crate lives two directories below the repository root")
    }

    fn members() -> BTreeSet<String> {
        read_workspace_members(&repo_root())
            .expect("the repository root declares a cargo workspace")
    }

    fn dockerfile_bodies() -> Vec<(PathBuf, String)> {
        let root = repo_root();
        find_dockerfiles(&root)
            .into_iter()
            .map(|df| {
                let body = fs::read_to_string(&df)
                    .unwrap_or_else(|e| panic!("read {}: {e}", df.display()));
                let rel = df.strip_prefix(&root).unwrap_or(&df).to_path_buf();
                (rel, body)
            })
            .collect()
    }

    /// Dockerfiles that build the workspace through `COPY` rather than bind mounts.
    fn copying_dockerfiles() -> Vec<(PathBuf, String)> {
        let members = members();
        dockerfile_bodies()
            .into_iter()
            .filter(|(_, body)| {
                workspace_coverage(body, &members)
                    .is_some_and(|c| c.mechanism == SourceMechanism::Copy)
            })
            .collect()
    }

    #[test]
    fn the_repository_passes_every_check() {
        let issues = run(&repo_root()).expect("the guard runs over the repository");
        assert_eq!(
            issues, 0,
            "the repository must satisfy the Dockerfile guard"
        );
    }

    #[test]
    fn workspace_members_are_covered_whatever_the_mechanism() {
        let members = members();
        let mut checked = 0;
        for (rel, body) in dockerfile_bodies() {
            let Some(coverage) = workspace_coverage(&body, &members) else {
                continue;
            };
            assert!(
                coverage.missing.is_empty(),
                "{} builds the workspace but does not provide {:?}",
                rel.display(),
                coverage.missing
            );
            checked += 1;
        }
        assert!(checked > 0, "no Dockerfile builds the cargo workspace");
    }

    #[test]
    fn a_member_dropped_from_a_copying_dockerfile_is_reported() {
        let members = members();
        let copying = copying_dockerfiles();
        assert!(
            !copying.is_empty(),
            "no Dockerfile builds the workspace through COPY, the check would be untested"
        );

        for (rel, body) in copying {
            let provided_members: Vec<String> = body
                .lines()
                .flat_map(copy_sources_of_line)
                .map(|s| normalize_source(&s))
                .filter(|s| members.contains(s))
                .collect();
            let dropped = provided_members
                .first()
                .unwrap_or_else(|| panic!("{} copies no workspace member", rel.display()))
                .clone();

            let mutated: String = body
                .lines()
                .filter(|line| {
                    !copy_sources_of_line(line)
                        .iter()
                        .any(|s| normalize_source(s) == dropped)
                })
                .map(|l| format!("{l}\n"))
                .collect();

            let coverage = workspace_coverage(&mutated, &members)
                .unwrap_or_else(|| panic!("{} still builds the workspace", rel.display()));
            assert_eq!(
                coverage.missing,
                vec![dropped.clone()],
                "{} lost `{dropped}` and the guard stayed silent",
                rel.display()
            );
            assert_eq!(
                coverage.mechanism.instruction_for(&dropped),
                format!("COPY {dropped} {dropped}"),
                "the guard must spell out the exact line to add back"
            );

            let restored = workspace_coverage(&body, &members)
                .unwrap_or_else(|| panic!("{} builds the workspace", rel.display()));
            assert!(
                restored.missing.is_empty(),
                "{} is complete once `{dropped}` is back",
                rel.display()
            );
        }
    }

    #[test]
    fn a_crate_built_from_its_own_context_is_left_alone() {
        let members = members();
        let self_contained: Vec<PathBuf> = dockerfile_bodies()
            .into_iter()
            .filter(|(rel, body)| {
                let owning_member = rel.parent().map(|p| p.to_string_lossy().to_string());
                let provided = provided_paths(body);
                owning_member.is_some_and(|m| members.contains(&m))
                    && provided.contains_key("Cargo.toml")
                    && !members.iter().any(|m| provided.contains_key(m.as_str()))
            })
            .map(|(rel, _)| rel)
            .collect();
        assert!(
            !self_contained.is_empty(),
            "no crate is built from its own context, the false-positive guard is untested"
        );

        let root = repo_root();
        for rel in self_contained {
            let body = fs::read_to_string(root.join(&rel))
                .unwrap_or_else(|e| panic!("read {}: {e}", rel.display()));
            assert!(
                workspace_coverage(&body, &members).is_none(),
                "{} builds from its own context and must not be asked for workspace members",
                rel.display()
            );
        }
    }
}

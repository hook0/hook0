use std::collections::BTreeSet;
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
        issues += check_workspace_mounts(rel, &body, &members);
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

fn check_workspace_mounts(rel: &Path, body: &str, members: &BTreeSet<String>) -> u32 {
    let mounts = extract_bind_sources(body);
    let mounts_cargo_toml = mounts.contains("Cargo.toml");
    let mounts_any_member = members.iter().any(|m| mounts.contains(m));
    if !mounts_cargo_toml || !mounts_any_member {
        return 0;
    }
    let missing: Vec<&String> = members
        .iter()
        .filter(|m| !mounts.contains(m.as_str()))
        .collect();
    if missing.is_empty() {
        println!(
            "[OK]   {} mounts all {} workspace members",
            rel.display(),
            members.len()
        );
        return 0;
    }
    eprintln!(
        "[FAIL] {} declares cargo workspace usage but is missing bind mounts for:",
        rel.display()
    );
    for m in &missing {
        eprintln!("         - {m}");
    }
    eprintln!(
        "       cargo refuses to build because the workspace Cargo.toml lists these as members."
    );
    eprintln!("       Add for each: --mount=type=bind,source=<member>,target=<member>");
    1
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
        let t = line.trim();
        let rest = match t.strip_prefix("COPY ").or_else(|| t.strip_prefix("ADD ")) {
            Some(r) => r,
            None => continue,
        };
        let parts: Vec<&str> = rest.split_whitespace().collect();
        let has_from_flag = parts.iter().any(|p| p.starts_with("--from="));
        if has_from_flag {
            continue;
        }
        let non_flag: Vec<&str> = parts.into_iter().filter(|p| !p.starts_with("--")).collect();
        if non_flag.len() < 2 {
            continue;
        }
        for src in &non_flag[..non_flag.len() - 1] {
            out.push(src.to_string());
        }
    }
    out
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
        let normalized = src.trim_start_matches("./").trim_end_matches('/');
        if normalized.is_empty() || normalized == "." {
            continue;
        }
        let head = normalized.split('/').next().unwrap_or(normalized);
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

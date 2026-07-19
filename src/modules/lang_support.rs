//! Shared helpers for the per-language project modules: manifest-file
//! reading and lightweight parsing, no full TOML/XML dependency.

use crate::context::Context;
use std::fs;
use std::path::Path;

/// The directory every language module detects against: the project root
/// reported by Claude Code, falling back to cwd outside a workspace.
pub fn project_dir(context: &Context) -> Option<&Path> {
    context
        .workspace
        .as_ref()
        .and_then(|w| w.project_dir.as_deref())
        .or(context.cwd.as_deref())
        .map(Path::new)
}

pub fn read(dir: &Path, relative: &str) -> Option<String> {
    fs::read_to_string(dir.join(relative)).ok()
}

pub fn value_after(content: &str, prefix: &str) -> Option<String> {
    content
        .lines()
        .find(|l| l.starts_with(prefix))
        .map(|l| l[prefix.len()..].trim().to_string())
}

pub fn between_quotes(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let end = start + line[start..].find('"')?;
    Some(line[start..end].to_string())
}

pub fn between_tags(content: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = content.find(&open)? + open.len();
    let end = start + content[start..].find(&close)?;
    Some(content[start..end].trim().to_string())
}

pub fn strip_semver_prefix(spec: &str) -> String {
    spec.trim_start_matches(['^', '~', '>', '=', ' '])
        .to_string()
}

pub fn first_with_extension(dir: &Path, extension: &str) -> Option<std::path::PathBuf> {
    let mut matches: Vec<_> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == extension))
        .collect();
    matches.sort();
    matches.into_iter().next()
}

/// Unity marks its project root with both of these directories.
pub fn is_unity_project(dir: &Path) -> bool {
    dir.join("Assets").is_dir() && dir.join("ProjectSettings").is_dir()
}

/// Extracts the first version-looking token from `--version`-style output:
/// `rustc 1.85.0 (…)` → `1.85.0`, `go version go1.22.1 linux/amd64` →
/// `1.22.1`, `v20.11.0` → `20.11.0`, `ruby 3.3.0p0 (…)` → `3.3.0`.
/// Tool-name prefixes are trimmed and build suffixes cut at the first
/// non-`[0-9.]` character; a match must contain a dot so dates and
/// revision hashes don't qualify.
pub fn parse_version_output(output: &str) -> Option<String> {
    output.split_whitespace().find_map(|token| {
        let token = token.trim_start_matches(|c: char| c.is_ascii_alphabetic());
        let version: String = token
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        (version.contains('.') && version.starts_with(|c: char| c.is_ascii_digit()))
            .then_some(version)
    })
}

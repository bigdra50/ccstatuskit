//! Project type icon + version, detected from marker files in the project
//! directory. Detection order is ported from the original bash statusline;
//! Unity wins over plain C#, package.json frameworks win over plain Node.

use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;
use std::fs;
use std::path::Path;

const UNITY: &str = "\u{e721}";
const REACT: &str = "\u{e7ba}";
const VUE: &str = "\u{f0844}"; // nf-md-vuejs (v2's U+FD42 range was removed in Nerd Fonts v3)
const NEXTJS: &str = "\u{f0e01}";
const TYPESCRIPT: &str = "\u{e628}";
const NODE: &str = "\u{e719}";
const RUST: &str = "\u{e7a8}";
const GO: &str = "\u{e724}";
const PYTHON: &str = "\u{f0320}";
const CSHARP: &str = "\u{f0aae}";
const RUBY: &str = "\u{f0d2d}";
const JAVA: &str = "\u{e738}";
const KOTLIN: &str = "\u{f10fe}";
const PHP: &str = "\u{f031f}";
const SWIFT: &str = "\u{e755}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = context
        .workspace
        .as_ref()
        .and_then(|w| w.project_dir.as_deref())
        .or(context.cwd.as_deref())?;
    let (icon, version) = detect(Path::new(dir))?;
    let text = match version {
        Some(version) => format!("{icon} {version}"),
        None => icon.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

fn detect(dir: &Path) -> Option<(&'static str, Option<String>)> {
    if dir.join("Assets").is_dir() && dir.join("ProjectSettings").is_dir() {
        let version = read(dir, "ProjectSettings/ProjectVersion.txt")
            .and_then(|s| value_after(&s, "m_EditorVersion:"));
        return Some((UNITY, version));
    }
    if let Some(pkg) = read(dir, "package.json") {
        return Some(node_family(dir, &pkg));
    }
    if let Some(cargo) = read(dir, "Cargo.toml") {
        let version = cargo
            .lines()
            .find(|l| l.trim_start().starts_with("version"))
            .and_then(between_quotes);
        return Some((RUST, version));
    }
    if let Some(gomod) = read(dir, "go.mod") {
        let version = gomod
            .lines()
            .find(|l| l.starts_with("go "))
            .map(|l| l[3..].trim().to_string());
        return Some((GO, version));
    }
    if dir.join("pyproject.toml").is_file()
        || dir.join("setup.py").is_file()
        || dir.join("requirements.txt").is_file()
    {
        let version = read(dir, ".python-version").map(|s| s.trim().to_string());
        return Some((PYTHON, version));
    }
    if let Some(csproj) = first_with_extension(dir, "csproj") {
        let version = fs::read_to_string(csproj)
            .ok()
            .and_then(|s| between_tags(&s, "TargetFramework"));
        return Some((CSHARP, version));
    }
    if dir.join("Gemfile").is_file() {
        let version = read(dir, ".ruby-version").map(|s| s.trim().to_string());
        return Some((RUBY, version));
    }
    if let Some(pom) = read(dir, "pom.xml") {
        return Some((JAVA, between_tags(&pom, "java.version")));
    }
    if dir.join("build.gradle").is_file() || dir.join("build.gradle.kts").is_file() {
        return Some((KOTLIN, None));
    }
    if let Some(composer) = read(dir, "composer.json") {
        let version = serde_json::from_str::<serde_json::Value>(&composer)
            .ok()
            .and_then(|v| {
                v.pointer("/require/php")
                    .and_then(|p| p.as_str())
                    .map(strip_semver_prefix)
            });
        return Some((PHP, version));
    }
    if dir.join("Package.swift").is_file() {
        return Some((SWIFT, None));
    }
    None
}

fn node_family(dir: &Path, pkg: &str) -> (&'static str, Option<String>) {
    let json: serde_json::Value = match serde_json::from_str(pkg) {
        Ok(json) => json,
        Err(_) => return (NODE, None),
    };
    let dep = |name: &str| -> Option<String> {
        json.pointer(&format!("/dependencies/{name}"))
            .or_else(|| json.pointer(&format!("/devDependencies/{name}")))
            .and_then(|v| v.as_str())
            .map(strip_semver_prefix)
    };
    if let Some(version) = dep("react") {
        return (REACT, Some(version));
    }
    if let Some(version) = dep("vue") {
        return (VUE, Some(version));
    }
    if let Some(version) = dep("next") {
        return (NEXTJS, Some(version));
    }
    if dir.join("tsconfig.json").is_file() {
        return (TYPESCRIPT, dep("typescript"));
    }
    let engine = json
        .pointer("/engines/node")
        .and_then(|v| v.as_str())
        .map(strip_semver_prefix);
    (NODE, engine)
}

fn read(dir: &Path, relative: &str) -> Option<String> {
    fs::read_to_string(dir.join(relative)).ok()
}

fn value_after(content: &str, prefix: &str) -> Option<String> {
    content
        .lines()
        .find(|l| l.starts_with(prefix))
        .map(|l| l[prefix.len()..].trim().to_string())
}

fn between_quotes(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let end = start + line[start..].find('"')?;
    Some(line[start..end].to_string())
}

fn between_tags(content: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = content.find(&open)? + open.len();
    let end = start + content[start..].find(&close)?;
    Some(content[start..end].trim().to_string())
}

fn strip_semver_prefix(spec: &str) -> String {
    spec.trim_start_matches(['^', '~', '>', '=', ' '])
        .to_string()
}

fn first_with_extension(dir: &Path, extension: &str) -> Option<std::path::PathBuf> {
    let mut matches: Vec<_> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == extension))
        .collect();
    matches.sort();
    matches.into_iter().next()
}

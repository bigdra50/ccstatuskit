//! The project's own declared version, from its manifest — starship's
//! `package` module. First match wins: `Cargo.toml`, `package.json`,
//! `pyproject.toml`, `composer.json`, `pom.xml`. This is deliberately a
//! separate module from the per-language ones, which show the toolchain
//! version instead.

use crate::context::Context;
use crate::modules::lang_support::{between_quotes, between_tags, project_dir, read};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;
use std::path::Path;

const ICON: &str = "\u{f03d7}"; // nf-md-package_variant_closed

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let version = cargo_toml(dir)
        .or_else(|| package_json(dir))
        .or_else(|| pyproject_toml(dir))
        .or_else(|| composer_json(dir))
        .or_else(|| pom_xml(dir))?;
    Some(Segment::styled(format!("{ICON} {version}"), theme::ORANGE))
}

fn cargo_toml(dir: &Path) -> Option<String> {
    read(dir, "Cargo.toml")?
        .lines()
        .find(|l| l.trim_start().starts_with("version"))
        .and_then(between_quotes)
}

fn package_json(dir: &Path) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(&read(dir, "package.json")?)
        .ok()?
        .pointer("/version")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn pyproject_toml(dir: &Path) -> Option<String> {
    // Matches both PEP 621 `[project]` and `[tool.poetry]` version lines.
    read(dir, "pyproject.toml")?
        .lines()
        .find(|l| l.trim_start().starts_with("version"))
        .and_then(between_quotes)
}

fn composer_json(dir: &Path) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(&read(dir, "composer.json")?)
        .ok()?
        .pointer("/version")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn pom_xml(dir: &Path) -> Option<String> {
    // Naive first-<version> scan; a <parent> block before the project's
    // own version can shadow it, which is acceptable for a statusline.
    between_tags(&read(dir, "pom.xml")?, "version")
}

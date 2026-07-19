//! Node project icon + version, from `package.json`. React/Vue/Next/
//! TypeScript win over plain Node and show that framework's dependency
//! version (icon and number always refer to the same thing); the plain
//! Node case shows the actual runtime version from `node --version`,
//! falling back to the `engines.node` constraint.

use crate::context::Context;
use crate::modules::lang_support::{parse_version_output, project_dir, read, strip_semver_prefix};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;
use std::path::Path;

const REACT: &str = "\u{e7ba}";
const VUE: &str = "\u{f0844}"; // nf-md-vuejs (v2's U+FD42 range was removed in Nerd Fonts v3)
const NEXTJS: &str = "\u{f0e01}";
const TYPESCRIPT: &str = "\u{e628}";
const NODE: &str = "\u{e719}";

pub fn render(context: &Context, probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let pkg = read(dir, "package.json")?;
    let (icon, version) = detect(dir, &pkg, probes);
    let text = match version {
        Some(version) => format!("{icon} {version}"),
        None => icon.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

fn detect(dir: &Path, pkg: &str, probes: &dyn Probes) -> (&'static str, Option<String>) {
    let json: serde_json::Value = match serde_json::from_str(pkg) {
        Ok(json) => json,
        Err(_) => {
            return (
                NODE,
                node_runtime_version(dir, probes, &serde_json::Value::Null),
            );
        }
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
    (NODE, node_runtime_version(dir, probes, &json))
}

fn node_runtime_version(
    dir: &Path,
    probes: &dyn Probes,
    json: &serde_json::Value,
) -> Option<String> {
    probes
        .run("node", &["--version"], &[], Some(dir))
        .as_deref()
        .and_then(parse_version_output)
        .or_else(|| {
            json.pointer("/engines/node")
                .and_then(|v| v.as_str())
                .map(strip_semver_prefix)
        })
}

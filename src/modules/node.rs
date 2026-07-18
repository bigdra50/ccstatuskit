//! Node project icon + version, from `package.json`. React/Vue/Next/
//! TypeScript win over plain Node, matching a project's actual framework.

use crate::context::Context;
use crate::modules::lang_support::{project_dir, read, strip_semver_prefix};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;
use std::path::Path;

const REACT: &str = "\u{e7ba}";
const VUE: &str = "\u{f0844}"; // nf-md-vuejs (v2's U+FD42 range was removed in Nerd Fonts v3)
const NEXTJS: &str = "\u{f0e01}";
const TYPESCRIPT: &str = "\u{e628}";
const NODE: &str = "\u{e719}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let pkg = read(dir, "package.json")?;
    let (icon, version) = detect(dir, &pkg);
    let text = match version {
        Some(version) => format!("{icon} {version}"),
        None => icon.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

fn detect(dir: &Path, pkg: &str) -> (&'static str, Option<String>) {
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

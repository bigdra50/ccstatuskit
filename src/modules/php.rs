//! PHP project icon + required version, from `composer.json`.

use crate::context::Context;
use crate::modules::lang_support::{project_dir, read, strip_semver_prefix};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{f031f}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let composer = read(dir, "composer.json")?;
    let version = serde_json::from_str::<serde_json::Value>(&composer)
        .ok()
        .and_then(|v| {
            v.pointer("/require/php")
                .and_then(|p| p.as_str())
                .map(strip_semver_prefix)
        });
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

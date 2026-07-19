//! PHP icon + interpreter version, from `php --version`; falls back to
//! the `require.php` constraint in `composer.json`.

use crate::context::Context;
use crate::modules::lang_support::{parse_version_output, project_dir, read, strip_semver_prefix};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{f031f}";

pub fn render(context: &Context, probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let composer = read(dir, "composer.json")?;
    let version = probes
        .run("php", &["--version"], &[], Some(dir))
        .as_deref()
        .and_then(parse_version_output)
        .or_else(|| {
            serde_json::from_str::<serde_json::Value>(&composer)
                .ok()
                .and_then(|v| {
                    v.pointer("/require/php")
                        .and_then(|p| p.as_str())
                        .map(strip_semver_prefix)
                })
        });
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

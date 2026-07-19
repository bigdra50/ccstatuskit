//! Ruby icon + interpreter version, from `ruby --version`; falls back to
//! the `.ruby-version` pin when the binary isn't available.

use crate::context::Context;
use crate::modules::lang_support::{parse_version_output, project_dir, read};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{f0d2d}";

pub fn render(context: &Context, probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    if !dir.join("Gemfile").is_file() {
        return None;
    }
    let version = probes
        .run("ruby", &["--version"], &[], Some(dir))
        .as_deref()
        .and_then(parse_version_output)
        .or_else(|| read(dir, ".ruby-version").map(|s| s.trim().to_string()));
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

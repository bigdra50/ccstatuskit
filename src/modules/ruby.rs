//! Ruby project icon + pinned version, from `.ruby-version`.

use crate::context::Context;
use crate::modules::lang_support::{project_dir, read};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{f0d2d}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    if !dir.join("Gemfile").is_file() {
        return None;
    }
    let version = read(dir, ".ruby-version").map(|s| s.trim().to_string());
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

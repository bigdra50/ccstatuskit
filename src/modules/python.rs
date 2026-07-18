//! Python project icon + pinned version, from `.python-version`.

use crate::context::Context;
use crate::modules::lang_support::{project_dir, read};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{f0320}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let is_python = dir.join("pyproject.toml").is_file()
        || dir.join("setup.py").is_file()
        || dir.join("requirements.txt").is_file();
    if !is_python {
        return None;
    }
    let version = read(dir, ".python-version").map(|s| s.trim().to_string());
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

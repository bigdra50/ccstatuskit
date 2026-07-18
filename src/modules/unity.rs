//! Unity project icon + editor version, from `ProjectSettings/ProjectVersion.txt`.

use crate::context::Context;
use crate::modules::lang_support::{is_unity_project, project_dir, read, value_after};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{e721}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    if !is_unity_project(dir) {
        return None;
    }
    let version = read(dir, "ProjectSettings/ProjectVersion.txt")
        .and_then(|s| value_after(&s, "m_EditorVersion:"));
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

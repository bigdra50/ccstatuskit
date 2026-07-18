//! .NET project icon + target framework, from the first `*.csproj` found.
//! Hidden inside a Unity project, which generates its own `.csproj` files.

use crate::context::Context;
use crate::modules::lang_support::{
    between_tags, first_with_extension, is_unity_project, project_dir,
};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;
use std::fs;

const ICON: &str = "\u{f0aae}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    if is_unity_project(dir) {
        return None;
    }
    let csproj = first_with_extension(dir, "csproj")?;
    let version = fs::read_to_string(csproj)
        .ok()
        .and_then(|s| between_tags(&s, "TargetFramework"));
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

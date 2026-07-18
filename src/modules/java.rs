//! Java project icon + language version, from `pom.xml`.

use crate::context::Context;
use crate::modules::lang_support::{between_tags, project_dir, read};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{e738}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let pom = read(dir, "pom.xml")?;
    let version = between_tags(&pom, "java.version");
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

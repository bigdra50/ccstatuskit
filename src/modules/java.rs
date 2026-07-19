//! Java icon + JDK version, from `java --version` (JDK 9+, stdout);
//! falls back to the `java.version` property in `pom.xml`.

use crate::context::Context;
use crate::modules::lang_support::{between_tags, parse_version_output, project_dir, read};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{e738}";

pub fn render(context: &Context, probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let pom = read(dir, "pom.xml")?;
    let version = probes
        .run("java", &["--version"], &[], Some(dir))
        .as_deref()
        .and_then(parse_version_output)
        .or_else(|| between_tags(&pom, "java.version"));
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

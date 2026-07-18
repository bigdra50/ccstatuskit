//! Go module icon + language version, from `go.mod`.

use crate::context::Context;
use crate::modules::lang_support::{project_dir, read};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{e724}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let gomod = read(dir, "go.mod")?;
    let version = gomod
        .lines()
        .find(|l| l.starts_with("go "))
        .map(|l| l[3..].trim().to_string());
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

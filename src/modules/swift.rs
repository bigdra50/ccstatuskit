//! Swift package icon, from `Package.swift`.

use crate::context::Context;
use crate::modules::lang_support::project_dir;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{e755}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    if dir.join("Package.swift").is_file() {
        Some(Segment::styled(ICON, theme::WHITE))
    } else {
        None
    }
}

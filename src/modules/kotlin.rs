//! Kotlin/Gradle project icon, from `build.gradle` or `build.gradle.kts`.

use crate::context::Context;
use crate::modules::lang_support::project_dir;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{f10fe}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    if dir.join("build.gradle").is_file() || dir.join("build.gradle.kts").is_file() {
        Some(Segment::styled(ICON, theme::WHITE))
    } else {
        None
    }
}

//! Open pull request, from `pr.number` / `pr.review_state`.

use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{ea64}"; // nf-oct-git_pull_request

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let pr = context.pr.as_ref()?;
    let number = pr.number?;
    let style = match pr.review_state.as_deref() {
        Some("approved") => theme::GREEN,
        Some("changes_requested") => theme::RED,
        _ => theme::YELLOW,
    };
    Some(Segment::styled(format!("{ICON} #{number}"), style))
}

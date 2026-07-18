//! Active subagent name, from `agent.name`.

use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{ee0d}"; // nf-fa-robot

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let name = context.agent.as_ref()?.name.as_deref()?;
    if name.trim().is_empty() {
        return None;
    }
    Some(Segment::styled(format!("{ICON} {name}"), theme::MAGENTA))
}

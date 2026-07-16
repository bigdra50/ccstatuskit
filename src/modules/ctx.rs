use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let pct = context.context_window.as_ref()?.used_percentage?;
    let pct = pct as u32;
    let style = if pct >= 60 {
        theme::RED
    } else if pct >= 40 {
        theme::YELLOW
    } else {
        theme::GREEN
    };
    Some(Segment::styled(format!("CTX {pct}%"), style))
}

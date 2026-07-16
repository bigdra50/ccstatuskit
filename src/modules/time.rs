use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

pub fn render(context: &Context, probes: &dyn Probes) -> Option<Segment> {
    let (hour, minute) = probes.local_hm();
    let mut text = format!("\u{f0954} {hour:02}:{minute:02}");
    if let Some(ms) = context.cost.as_ref().and_then(|c| c.total_duration_ms) {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1000;
        if minutes > 0 {
            text.push_str(&format!(" ({minutes}m)"));
        } else if seconds > 0 {
            text.push_str(&format!(" ({seconds}s)"));
        }
    }
    Some(Segment::styled(text, theme::ORANGE))
}

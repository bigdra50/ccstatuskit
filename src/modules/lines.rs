//! Session-wide edit stats, from `cost.total_lines_added` / `total_lines_removed`.

use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let cost = context.cost.as_ref()?;
    let added = cost.total_lines_added.unwrap_or(0);
    let removed = cost.total_lines_removed.unwrap_or(0);
    if added == 0 && removed == 0 {
        return None;
    }
    Some(Segment::styled(format!("+{added} -{removed}"), theme::CYAN))
}

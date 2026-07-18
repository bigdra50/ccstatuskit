//! Session cost in USD, from `cost.total_cost_usd`.

use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{f155}"; // nf-fa-dollar

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let usd = context.cost.as_ref()?.total_cost_usd?;
    let style = if usd >= 5.0 {
        theme::RED
    } else if usd >= 1.0 {
        theme::YELLOW
    } else {
        theme::GREEN
    };
    Some(Segment::styled(format!("{ICON} ${usd:.2}"), style))
}

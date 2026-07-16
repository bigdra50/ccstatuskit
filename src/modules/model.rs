use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let name = context.model.as_ref()?.display_name.as_ref()?;
    let family = name.to_lowercase();
    let style = if family.contains("sonnet") {
        theme::BLUE
    } else if family.contains("opus") {
        theme::MAGENTA
    } else if family.contains("haiku") {
        theme::GREEN
    } else {
        theme::WHITE
    };
    Some(Segment::styled(format!("\u{f06a9} {name}"), style))
}

use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

pub fn render(_context: &Context, probes: &dyn Probes) -> Option<Segment> {
    let mem = probes.memory()?;
    if mem.total_kb == 0 {
        return None;
    }
    let used_percent = mem.used_kb * 100 / mem.total_kb;
    // Round to the nearest GiB, matching the original bash statusline.
    let used_gb = (mem.used_kb + 524_288) / 1_048_576;
    let total_gb = (mem.total_kb + 524_288) / 1_048_576;
    let style = if used_percent > 80 {
        theme::RED
    } else if used_percent > 60 {
        theme::YELLOW
    } else {
        theme::GREEN
    };
    Some(Segment::styled(
        format!("\u{f035b} {used_gb}/{total_gb}GB"),
        style,
    ))
}

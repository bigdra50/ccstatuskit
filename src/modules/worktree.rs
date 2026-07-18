//! Git worktree context, from `worktree.name` / `worktree.branch`.

use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{ea63}"; // nf-dev-repo_forked

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let worktree = context.worktree.as_ref()?;
    let name = worktree.name.as_deref()?;
    if name.trim().is_empty() {
        return None;
    }
    let text = match worktree.branch.as_deref() {
        Some(branch) if !branch.trim().is_empty() => format!("{ICON} {name}:{branch}"),
        _ => format!("{ICON} {name}"),
    };
    Some(Segment::styled(text, theme::YELLOW))
}

use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let workspace = context.workspace.as_ref();
    let current = workspace
        .and_then(|w| w.current_dir.as_deref())
        .or(context.cwd.as_deref())?;
    let project = workspace.and_then(|w| w.project_dir.as_deref());

    let display = match project {
        Some(project) if current.starts_with(project) => {
            let relative = current[project.len()..].trim_start_matches('/');
            if relative.is_empty() {
                basename(project)
            } else {
                relative
            }
        }
        _ => basename(current),
    };
    Some(Segment::styled(
        format!("\u{f0256} {display}"),
        theme::GREEN,
    ))
}

fn basename(path: &str) -> &str {
    let trimmed = path.trim_end_matches(['/', '\\']);
    trimmed
        .rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(trimmed)
}

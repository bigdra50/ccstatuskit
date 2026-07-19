//! Go icon + toolchain version, from `go version`; falls back to the
//! `go` directive in `go.mod` when the binary isn't available.

use crate::context::Context;
use crate::modules::lang_support::{parse_version_output, project_dir, read};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{e724}";

pub fn render(context: &Context, probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let gomod = read(dir, "go.mod")?;
    let version = probes
        .run("go", &["version"], &[], Some(dir))
        .as_deref()
        .and_then(parse_version_output)
        .or_else(|| {
            gomod
                .lines()
                .find(|l| l.starts_with("go "))
                .map(|l| l[3..].trim().to_string())
        });
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

//! Rust icon + active toolchain version, from `rustc --version` run in the
//! project directory so rustup per-directory overrides apply. Detection:
//! `Cargo.toml`. The crate's own declared version lives in `$package`.

use crate::context::Context;
use crate::modules::lang_support::{parse_version_output, project_dir};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{e7a8}";

pub fn render(context: &Context, probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    if !dir.join("Cargo.toml").is_file() {
        return None;
    }
    let version = probes
        .run("rustc", &["--version"], &[], Some(dir))
        .as_deref()
        .and_then(parse_version_output);
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

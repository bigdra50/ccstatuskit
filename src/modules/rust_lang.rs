//! Rust crate icon + version, from `Cargo.toml`.

use crate::context::Context;
use crate::modules::lang_support::{between_quotes, project_dir, read};
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;

const ICON: &str = "\u{e7a8}";

pub fn render(context: &Context, _probes: &dyn Probes) -> Option<Segment> {
    let dir = project_dir(context)?;
    let cargo = read(dir, "Cargo.toml")?;
    let version = cargo
        .lines()
        .find(|l| l.trim_start().starts_with("version"))
        .and_then(between_quotes);
    let text = match version {
        Some(version) => format!("{ICON} {version}"),
        None => ICON.to_string(),
    };
    Some(Segment::styled(text, theme::WHITE))
}

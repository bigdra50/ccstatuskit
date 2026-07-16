mod common;

use anstyle::{Color, RgbColor, Style};
use ccstatuskit::config::{Config, UsageConfig};
use ccstatuskit::context::Context;
use ccstatuskit::modules::BuiltinModule;
use common::FakeProbes;

fn rgb(r: u8, g: u8, b: u8) -> Style {
    Style::new().fg_color(Some(Color::Rgb(RgbColor(r, g, b))))
}

const GRAY: (u8, u8, u8) = (0x75, 0x71, 0x5E);
const ORANGE: (u8, u8, u8) = (0xFD, 0x97, 0x1F);
const RED: (u8, u8, u8) = (0xF9, 0x26, 0x72);

fn ctx(json: &str) -> Context {
    Context::from_stdin_json(json.to_string())
}

fn full_ctx() -> Context {
    ctx(&std::fs::read_to_string(format!(
        "{}/tests/fixtures/full.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap())
}

fn render(context: &Context, usage: UsageConfig) -> Option<ccstatuskit::segment::Segment> {
    let config = Config {
        usage,
        ..Default::default()
    };
    BuiltinModule::Usage.render_with(context, &config, &FakeProbes::default())
}

fn no_refresh() -> UsageConfig {
    UsageConfig {
        auto_refresh: false,
        ..Default::default()
    }
}

// --- stdin rate_limits (official source) ---

#[test]
fn renders_stdin_rate_limits() {
    let seg = render(&full_ctx(), no_refresh()).unwrap();
    assert_eq!(seg.text, "5h 24% \u{b7} wk 41%");
    assert_eq!(seg.style, rgb(GRAY.0, GRAY.1, GRAY.2));
}

#[test]
fn warning_threshold_adds_mark_and_orange() {
    let context = ctx(r#"{"rate_limits":{"five_hour":{"used_percentage":85}}}"#);
    let seg = render(&context, no_refresh()).unwrap();
    assert_eq!(seg.text, "5h 85%\u{26a0}");
    assert_eq!(seg.style, rgb(ORANGE.0, ORANGE.1, ORANGE.2));
}

#[test]
fn critical_threshold_adds_mark_and_red() {
    let context = ctx(
        r#"{"rate_limits":{"five_hour":{"used_percentage":20},"seven_day":{"used_percentage":96}}}"#,
    );
    let seg = render(&context, no_refresh()).unwrap();
    assert_eq!(seg.text, "5h 20% \u{b7} wk 96%\u{26d4}");
    assert_eq!(seg.style, rgb(RED.0, RED.1, RED.2));
}

#[test]
fn only_filters_windows() {
    let usage = UsageConfig {
        only: vec!["seven_day".to_string()],
        ..no_refresh()
    };
    let seg = render(&full_ctx(), usage).unwrap();
    assert_eq!(seg.text, "wk 41%");
}

#[test]
fn hides_without_any_rate_limit_data() {
    let seg = render(&Context::default(), no_refresh());
    assert_eq!(seg, None);
}

// --- scoped quotas from the SWR cache (unofficial API, opt-in) ---

const CACHE_BODY: &str = r#"{"limits":[
  {"kind":"session","percent":12.0,"severity":"normal"},
  {"kind":"weekly_all","percent":48.2,"severity":"normal"},
  {"kind":"weekly_scoped","group":"weekly","percent":76.0,"severity":"warning",
   "scope":{"model":{"display_name":"Fable"}}}
]}"#;

fn scoped_config(dir: &std::path::Path) -> UsageConfig {
    UsageConfig {
        scoped: true,
        auto_refresh: false,
        cache_dir: Some(dir.to_path_buf()),
        ..Default::default()
    }
}

#[test]
fn scoped_renders_all_windows_from_cache() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("usage.json"), CACHE_BODY).unwrap();
    let seg = render(&full_ctx(), scoped_config(dir.path())).unwrap();
    assert_eq!(seg.text, "5h 12% \u{b7} wk 48% \u{b7} Fable 76%\u{26a0}");
    assert_eq!(seg.style, rgb(ORANGE.0, ORANGE.1, ORANGE.2));
}

#[test]
fn scoped_only_fable_shows_a_single_window() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("usage.json"), CACHE_BODY).unwrap();
    let usage = UsageConfig {
        only: vec!["fable".to_string()],
        ..scoped_config(dir.path())
    };
    let seg = render(&full_ctx(), usage).unwrap();
    assert_eq!(seg.text, "Fable 76%\u{26a0}");
}

#[test]
fn scoped_falls_back_to_stdin_when_cache_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let seg = render(&full_ctx(), scoped_config(dir.path())).unwrap();
    assert_eq!(seg.text, "5h 24% \u{b7} wk 41%");
}

#[test]
fn scoped_ignores_broken_cache_and_falls_back() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("usage.json"), "not json").unwrap();
    let seg = render(&full_ctx(), scoped_config(dir.path())).unwrap();
    assert_eq!(seg.text, "5h 24% \u{b7} wk 41%");
}

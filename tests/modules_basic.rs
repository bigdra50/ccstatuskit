mod common;

use anstyle::{Color, RgbColor, Style};
use ccstatuskit::context::Context;
use ccstatuskit::modules::BuiltinModule;
use ccstatuskit::segment::Segment;
use common::FakeProbes;

fn rgb(r: u8, g: u8, b: u8) -> Style {
    Style::new().fg_color(Some(Color::Rgb(RgbColor(r, g, b))))
}

fn ctx(json: &str) -> Context {
    Context::from_stdin_json(json.to_string())
}

// --- model ---

#[test]
fn model_shows_display_name_with_family_color() {
    let cases = [
        ("Opus", rgb(0xAE, 0x81, 0xFF)),
        ("Sonnet", rgb(0x66, 0xD9, 0xEF)),
        ("Haiku 4.5", rgb(0xA6, 0xE2, 0x2E)),
        ("Fable", rgb(0xF8, 0xF8, 0xF2)),
    ];
    for (name, style) in cases {
        let context = ctx(&format!(r#"{{"model":{{"display_name":"{name}"}}}}"#));
        let seg = BuiltinModule::Model
            .render(&context, &FakeProbes::default())
            .unwrap();
        assert_eq!(seg, Segment::styled(format!("\u{f06a9} {name}"), style));
    }
}

#[test]
fn model_hides_without_model_info() {
    let seg = BuiltinModule::Model.render(&Context::default(), &FakeProbes::default());
    assert_eq!(seg, None);
}

// --- directory ---

#[test]
fn directory_shows_path_relative_to_project() {
    let context =
        ctx(r#"{"workspace":{"current_dir":"/home/u/proj/src/app","project_dir":"/home/u/proj"}}"#);
    let seg = BuiltinModule::Directory
        .render(&context, &FakeProbes::default())
        .unwrap();
    assert_eq!(
        seg,
        Segment::styled("\u{f0256} src/app", rgb(0xA6, 0xE2, 0x2E))
    );
}

#[test]
fn directory_shows_project_basename_at_project_root() {
    let context =
        ctx(r#"{"workspace":{"current_dir":"/home/u/proj","project_dir":"/home/u/proj"}}"#);
    let seg = BuiltinModule::Directory
        .render(&context, &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f0256} proj");
}

#[test]
fn directory_outside_project_shows_current_basename() {
    let context = ctx(r#"{"workspace":{"current_dir":"/tmp/x","project_dir":"/home/u/proj"}}"#);
    let seg = BuiltinModule::Directory
        .render(&context, &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f0256} x");
}

#[test]
fn directory_falls_back_to_cwd() {
    let context = ctx(r#"{"cwd":"/a/b"}"#);
    let seg = BuiltinModule::Directory
        .render(&context, &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f0256} b");
}

// --- ctx (context window) ---

#[test]
fn ctx_colors_follow_usage_thresholds() {
    let cases = [
        (30.0, rgb(0xA6, 0xE2, 0x2E)),
        (45.5, rgb(0xE6, 0xDB, 0x74)),
        (72.0, rgb(0xF9, 0x26, 0x72)),
    ];
    for (pct, style) in cases {
        let context = ctx(&format!(
            r#"{{"context_window":{{"used_percentage":{pct}}}}}"#
        ));
        let seg = BuiltinModule::Ctx
            .render(&context, &FakeProbes::default())
            .unwrap();
        assert_eq!(seg, Segment::styled(format!("CTX {}%", pct as u32), style));
    }
}

#[test]
fn ctx_hides_without_percentage() {
    let seg = BuiltinModule::Ctx.render(&Context::default(), &FakeProbes::default());
    assert_eq!(seg, None);
}

// --- time ---

#[test]
fn time_shows_clock_with_minute_duration() {
    let probes = FakeProbes {
        hm: (14, 5),
        ..Default::default()
    };
    let context = ctx(r#"{"cost":{"total_duration_ms":125000}}"#);
    let seg = BuiltinModule::Time.render(&context, &probes).unwrap();
    assert_eq!(
        seg,
        Segment::styled("\u{f0954} 14:05 (2m)", rgb(0xFD, 0x97, 0x1F))
    );
}

#[test]
fn time_shows_seconds_when_under_a_minute() {
    let probes = FakeProbes {
        hm: (9, 30),
        ..Default::default()
    };
    let context = ctx(r#"{"cost":{"total_duration_ms":45000}}"#);
    let seg = BuiltinModule::Time.render(&context, &probes).unwrap();
    assert_eq!(seg.text, "\u{f0954} 09:30 (45s)");
}

#[test]
fn time_omits_duration_when_absent_or_zero() {
    let probes = FakeProbes {
        hm: (23, 59),
        ..Default::default()
    };
    let seg = BuiltinModule::Time
        .render(&Context::default(), &probes)
        .unwrap();
    assert_eq!(seg.text, "\u{f0954} 23:59");
}

// --- registry ---

#[test]
fn builtin_names_resolve() {
    for name in [
        "model",
        "directory",
        "git",
        "memory",
        "ctx",
        "time",
        "project",
        "usage",
    ] {
        assert!(BuiltinModule::from_name(name).is_some(), "missing: {name}");
    }
    assert!(BuiltinModule::from_name("nope").is_none());
}

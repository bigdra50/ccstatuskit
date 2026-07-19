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

// --- cost ---

#[test]
fn cost_shows_dollar_amount_with_thresholds() {
    let cases = [
        (0.42, rgb(0xA6, 0xE2, 0x2E)),
        (2.5, rgb(0xE6, 0xDB, 0x74)),
        (12.0, rgb(0xF9, 0x26, 0x72)),
    ];
    for (usd, style) in cases {
        let context = ctx(&format!(r#"{{"cost":{{"total_cost_usd":{usd}}}}}"#));
        let seg = BuiltinModule::Cost
            .render(&context, &FakeProbes::default())
            .unwrap();
        assert_eq!(seg, Segment::styled(format!("\u{f155} ${usd:.2}"), style));
    }
}

#[test]
fn cost_hides_without_data() {
    let seg = BuiltinModule::Cost.render(&Context::default(), &FakeProbes::default());
    assert_eq!(seg, None);
}

// --- lines ---

#[test]
fn lines_shows_added_and_removed() {
    let context = ctx(r#"{"cost":{"total_lines_added":120,"total_lines_removed":45}}"#);
    let seg = BuiltinModule::Lines
        .render(&context, &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "+120 -45");
}

#[test]
fn lines_hides_when_nothing_changed() {
    let context = ctx(r#"{"cost":{"total_lines_added":0,"total_lines_removed":0}}"#);
    let seg = BuiltinModule::Lines.render(&context, &FakeProbes::default());
    assert_eq!(seg, None);
}

// --- agent ---

#[test]
fn agent_shows_subagent_name() {
    let context = ctx(r#"{"agent":{"name":"Explore"}}"#);
    let seg = BuiltinModule::Agent
        .render(&context, &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{ee0d} Explore");
}

#[test]
fn agent_hides_without_active_agent() {
    let seg = BuiltinModule::Agent.render(&Context::default(), &FakeProbes::default());
    assert_eq!(seg, None);
}

// --- pr ---

#[test]
fn pr_colors_follow_review_state() {
    let cases = [
        ("approved", rgb(0xA6, 0xE2, 0x2E)),
        ("changes_requested", rgb(0xF9, 0x26, 0x72)),
        ("pending", rgb(0xE6, 0xDB, 0x74)),
    ];
    for (state, style) in cases {
        let context = ctx(&format!(
            r#"{{"pr":{{"number":42,"review_state":"{state}"}}}}"#
        ));
        let seg = BuiltinModule::Pr
            .render(&context, &FakeProbes::default())
            .unwrap();
        assert_eq!(seg, Segment::styled("\u{ea64} #42", style));
    }
}

#[test]
fn pr_hides_without_a_number() {
    let seg = BuiltinModule::Pr.render(&Context::default(), &FakeProbes::default());
    assert_eq!(seg, None);
}

// --- worktree ---

#[test]
fn worktree_shows_name_and_branch() {
    let context = ctx(r#"{"worktree":{"name":"feature-x","branch":"feature/x"}}"#);
    let seg = BuiltinModule::Worktree
        .render(&context, &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{ea63} feature-x:feature/x");
}

#[test]
fn worktree_hides_without_a_name() {
    let seg = BuiltinModule::Worktree.render(&Context::default(), &FakeProbes::default());
    assert_eq!(seg, None);
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
        "usage",
        "cost",
        "lines",
        "agent",
        "pr",
        "worktree",
        "package",
        "unity",
        "node",
        "rust",
        "go",
        "python",
        "dotnet",
        "ruby",
        "java",
        "kotlin",
        "php",
        "swift",
    ] {
        assert!(BuiltinModule::from_name(name).is_some(), "missing: {name}");
    }
    assert!(BuiltinModule::from_name("nope").is_none());
}

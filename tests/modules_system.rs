mod common;

use anstyle::{Color, RgbColor, Style};
use ccstatuskit::context::Context;
use ccstatuskit::modules::BuiltinModule;
use ccstatuskit::probes::MemInfo;
use common::FakeProbes;

fn rgb(r: u8, g: u8, b: u8) -> Style {
    Style::new().fg_color(Some(Color::Rgb(RgbColor(r, g, b))))
}

fn ctx(json: &str) -> Context {
    Context::from_stdin_json(json.to_string())
}

fn gb(gigabytes: u64) -> u64 {
    gigabytes * 1_048_576
}

// --- memory ---

#[test]
fn memory_formats_used_and_total_gigabytes() {
    let probes = FakeProbes {
        mem: Some(MemInfo {
            used_kb: gb(12),
            total_kb: gb(24),
        }),
        ..Default::default()
    };
    let seg = BuiltinModule::Memory
        .render(&Context::default(), &probes)
        .unwrap();
    assert_eq!(seg.text, "\u{f035b} 12/24GB");
    assert_eq!(seg.style, rgb(0xA6, 0xE2, 0x2E));
}

#[test]
fn memory_colors_follow_pressure_thresholds() {
    for (used, style) in [
        (gb(17), rgb(0xE6, 0xDB, 0x74)), // ~71%
        (gb(22), rgb(0xF9, 0x26, 0x72)), // ~92%
    ] {
        let probes = FakeProbes {
            mem: Some(MemInfo {
                used_kb: used,
                total_kb: gb(24),
            }),
            ..Default::default()
        };
        let seg = BuiltinModule::Memory
            .render(&Context::default(), &probes)
            .unwrap();
        assert_eq!(seg.style, style);
    }
}

#[test]
fn memory_hides_when_probe_has_nothing() {
    let seg = BuiltinModule::Memory.render(&Context::default(), &FakeProbes::default());
    assert_eq!(seg, None);
}

// --- project ---

fn project_ctx(dir: &std::path::Path) -> Context {
    ctx(&format!(
        r#"{{"workspace":{{"project_dir":{}}}}}"#,
        serde_json::to_string(dir.to_str().unwrap()).unwrap()
    ))
}

#[test]
fn project_detects_unity_with_editor_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("Assets")).unwrap();
    std::fs::create_dir(dir.path().join("ProjectSettings")).unwrap();
    std::fs::write(
        dir.path().join("ProjectSettings/ProjectVersion.txt"),
        "m_EditorVersion: 6000.0.32f1\nm_EditorVersionWithRevision: 6000.0.32f1 (x)\n",
    )
    .unwrap();
    let seg = BuiltinModule::Project
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e721} 6000.0.32f1");
}

#[test]
fn project_detects_rust_crate_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nversion = \"1.2.3\"\n",
    )
    .unwrap();
    let seg = BuiltinModule::Project
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e7a8} 1.2.3");
}

#[test]
fn project_detects_react_from_package_json() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"dependencies":{"react":"^18.2.0"}}"#,
    )
    .unwrap();
    let seg = BuiltinModule::Project
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e7ba} 18.2.0");
}

#[test]
fn project_detects_go_module() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("go.mod"),
        "module example.com/x\n\ngo 1.22\n",
    )
    .unwrap();
    let seg = BuiltinModule::Project
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e724} 1.22");
}

#[test]
fn project_hides_when_nothing_detected() {
    let dir = tempfile::tempdir().unwrap();
    let seg = BuiltinModule::Project.render(&project_ctx(dir.path()), &FakeProbes::default());
    assert_eq!(seg, None);
}

// --- git ---

fn git_ctx() -> Context {
    ctx(r#"{"workspace":{"current_dir":"/repo"}}"#)
}

fn git_probes() -> FakeProbes {
    FakeProbes::default().with_cmd("git rev-parse --git-dir", "/nonexistent/.git\n")
}

#[test]
fn git_clean_repo_renders_branch_with_check() {
    let probes = git_probes()
        .with_cmd("git branch --show-current", "main\n")
        .with_cmd("git status --porcelain", "");
    let seg = BuiltinModule::Git.render(&git_ctx(), &probes).unwrap();
    assert_eq!(seg.text, "\u{e0a0} main \u{2713}");
    assert_eq!(seg.style, rgb(0xA6, 0xE2, 0x2E));
}

#[test]
fn git_dirty_repo_shows_state_icons_in_yellow() {
    let probes = git_probes()
        .with_cmd("git branch --show-current", "main\n")
        .with_cmd(
            "git status --porcelain",
            "M  staged.rs\n M modified.rs\n?? new.rs\n",
        );
    let seg = BuiltinModule::Git.render(&git_ctx(), &probes).unwrap();
    assert_eq!(seg.text, "\u{e0a0} main \u{25cf}\u{270e}?");
    assert_eq!(seg.style, rgb(0xE6, 0xDB, 0x74));
}

#[test]
fn git_detached_head_shows_short_hash() {
    let probes = git_probes()
        .with_cmd("git branch --show-current", "")
        .with_cmd("git rev-parse --short HEAD", "abc1234\n")
        .with_cmd("git status --porcelain", "");
    let seg = BuiltinModule::Git.render(&git_ctx(), &probes).unwrap();
    assert_eq!(seg.text, "\u{f0e2c} abc1234 \u{2713}");
}

#[test]
fn git_shows_ahead_behind_counts() {
    let probes = git_probes()
        .with_cmd("git branch --show-current", "main\n")
        .with_cmd("git status --porcelain", "")
        .with_cmd("git rev-parse --abbrev-ref @{u}", "origin/main\n")
        .with_cmd(
            "git rev-list --left-right --count HEAD...origin/main",
            "2\t1\n",
        );
    let seg = BuiltinModule::Git.render(&git_ctx(), &probes).unwrap();
    assert_eq!(seg.text, "\u{e0a0} main \u{2713}\u{2191}2\u{2193}1");
}

#[test]
fn git_merge_in_progress_renders_red() {
    let gitdir = tempfile::tempdir().unwrap();
    std::fs::write(gitdir.path().join("MERGE_HEAD"), "abc\n").unwrap();
    let probes = FakeProbes::default()
        .with_cmd(
            "git rev-parse --git-dir",
            &format!("{}\n", gitdir.path().display()),
        )
        .with_cmd("git branch --show-current", "main\n")
        .with_cmd("git status --porcelain", "");
    let seg = BuiltinModule::Git.render(&git_ctx(), &probes).unwrap();
    assert_eq!(seg.text, "\u{e0a0} main \u{26a1}\u{2713}");
    assert_eq!(seg.style, rgb(0xF9, 0x26, 0x72));
}

#[test]
fn git_hides_outside_a_repository() {
    let probes = FakeProbes::default();
    let seg = BuiltinModule::Git.render(&git_ctx(), &probes);
    assert_eq!(seg, None);
}

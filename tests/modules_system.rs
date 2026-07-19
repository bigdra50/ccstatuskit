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

// --- per-language project modules ---

fn project_ctx(dir: &std::path::Path) -> Context {
    ctx(&format!(
        r#"{{"workspace":{{"project_dir":{}}}}}"#,
        serde_json::to_string(dir.to_str().unwrap()).unwrap()
    ))
}

#[test]
fn unity_detects_editor_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("Assets")).unwrap();
    std::fs::create_dir(dir.path().join("ProjectSettings")).unwrap();
    std::fs::write(
        dir.path().join("ProjectSettings/ProjectVersion.txt"),
        "m_EditorVersion: 6000.0.32f1\nm_EditorVersionWithRevision: 6000.0.32f1 (x)\n",
    )
    .unwrap();
    let seg = BuiltinModule::Unity
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e721} 6000.0.32f1");
}

#[test]
fn unity_hides_outside_a_unity_project() {
    let dir = tempfile::tempdir().unwrap();
    let seg = BuiltinModule::Unity.render(&project_ctx(dir.path()), &FakeProbes::default());
    assert_eq!(seg, None);
}

#[test]
fn rust_shows_toolchain_version_from_rustc() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nversion = \"1.2.3\"\n",
    )
    .unwrap();
    let probes =
        FakeProbes::default().with_cmd("rustc --version", "rustc 1.85.0 (4d91de4e4 2025-02-17)\n");
    let seg = BuiltinModule::Rust
        .render(&project_ctx(dir.path()), &probes)
        .unwrap();
    assert_eq!(seg.text, "\u{e7a8} 1.85.0");
}

#[test]
fn rust_shows_icon_only_without_rustc() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
    let seg = BuiltinModule::Rust
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e7a8}");
}

#[test]
fn node_detects_react_from_package_json() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"dependencies":{"react":"^18.2.0"}}"#,
    )
    .unwrap();
    let seg = BuiltinModule::Node
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e7ba} 18.2.0");
}

#[test]
fn node_shows_runtime_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
    let probes = FakeProbes::default().with_cmd("node --version", "v20.11.0\n");
    let seg = BuiltinModule::Node
        .render(&project_ctx(dir.path()), &probes)
        .unwrap();
    assert_eq!(seg.text, "\u{e719} 20.11.0");
}

#[test]
fn node_falls_back_to_engines_constraint() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"engines":{"node":">=20"}}"#,
    )
    .unwrap();
    let seg = BuiltinModule::Node
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e719} 20");
}

#[test]
fn go_shows_toolchain_version_from_go_binary() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("go.mod"),
        "module example.com/x\n\ngo 1.22\n",
    )
    .unwrap();
    let probes = FakeProbes::default().with_cmd("go version", "go version go1.22.1 linux/amd64\n");
    let seg = BuiltinModule::Go
        .render(&project_ctx(dir.path()), &probes)
        .unwrap();
    assert_eq!(seg.text, "\u{e724} 1.22.1");
}

#[test]
fn go_falls_back_to_gomod_directive() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("go.mod"),
        "module example.com/x\n\ngo 1.22\n",
    )
    .unwrap();
    let seg = BuiltinModule::Go
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e724} 1.22");
}

#[test]
fn python_shows_interpreter_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"x\"\n",
    )
    .unwrap();
    let probes = FakeProbes::default().with_cmd("python3 --version", "Python 3.12.1\n");
    let seg = BuiltinModule::Python
        .render(&project_ctx(dir.path()), &probes)
        .unwrap();
    assert_eq!(seg.text, "\u{f0320} 3.12.1");
}

#[test]
fn python_falls_back_to_pinned_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"x\"\n",
    )
    .unwrap();
    std::fs::write(dir.path().join(".python-version"), "3.12\n").unwrap();
    let seg = BuiltinModule::Python
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f0320} 3.12");
}

#[test]
fn dotnet_detects_target_framework() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("app.csproj"),
        "<Project><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>",
    )
    .unwrap();
    let seg = BuiltinModule::Dotnet
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f0aae} net8.0");
}

#[test]
fn dotnet_hides_inside_a_unity_project() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("Assets")).unwrap();
    std::fs::create_dir(dir.path().join("ProjectSettings")).unwrap();
    std::fs::write(dir.path().join("Assembly-CSharp.csproj"), "<Project/>").unwrap();
    let seg = BuiltinModule::Dotnet.render(&project_ctx(dir.path()), &FakeProbes::default());
    assert_eq!(seg, None);
}

#[test]
fn ruby_shows_interpreter_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Gemfile"),
        "source \"https://rubygems.org\"\n",
    )
    .unwrap();
    let probes = FakeProbes::default().with_cmd(
        "ruby --version",
        "ruby 3.3.6p108 (2024-11-05 revision 75015d4c1f) [arm64-darwin23]\n",
    );
    let seg = BuiltinModule::Ruby
        .render(&project_ctx(dir.path()), &probes)
        .unwrap();
    assert_eq!(seg.text, "\u{f0d2d} 3.3.6");
}

#[test]
fn ruby_falls_back_to_pinned_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Gemfile"),
        "source \"https://rubygems.org\"\n",
    )
    .unwrap();
    std::fs::write(dir.path().join(".ruby-version"), "3.3.0\n").unwrap();
    let seg = BuiltinModule::Ruby
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f0d2d} 3.3.0");
}

#[test]
fn java_shows_jdk_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("pom.xml"), "<project></project>").unwrap();
    let probes = FakeProbes::default().with_cmd(
        "java --version",
        "openjdk 21.0.2 2024-01-16\nOpenJDK Runtime Environment (build 21.0.2+13)\n",
    );
    let seg = BuiltinModule::Java
        .render(&project_ctx(dir.path()), &probes)
        .unwrap();
    assert_eq!(seg.text, "\u{e738} 21.0.2");
}

#[test]
fn java_falls_back_to_pom_language_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("pom.xml"),
        "<project><properties><java.version>21</java.version></properties></project>",
    )
    .unwrap();
    let seg = BuiltinModule::Java
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e738} 21");
}

#[test]
fn kotlin_detects_gradle_kts() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("build.gradle.kts"), "").unwrap();
    let seg = BuiltinModule::Kotlin
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f10fe}");
}

#[test]
fn php_shows_interpreter_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("composer.json"), r#"{"name":"x/x"}"#).unwrap();
    let probes = FakeProbes::default().with_cmd(
        "php --version",
        "PHP 8.3.2 (cli) (built: Jan 20 2024 14:16:20) (NTS)\n",
    );
    let seg = BuiltinModule::Php
        .render(&project_ctx(dir.path()), &probes)
        .unwrap();
    assert_eq!(seg.text, "\u{f031f} 8.3.2");
}

#[test]
fn php_falls_back_to_composer_constraint() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("composer.json"),
        r#"{"require":{"php":"^8.3"}}"#,
    )
    .unwrap();
    let seg = BuiltinModule::Php
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f031f} 8.3");
}

#[test]
fn swift_detects_package_manifest() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Package.swift"), "").unwrap();
    let seg = BuiltinModule::Swift
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{e755}");
}

// --- package (project's own declared version) ---

#[test]
fn package_shows_cargo_crate_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nversion = \"1.2.3\"\n",
    )
    .unwrap();
    let seg = BuiltinModule::Package
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f03d7} 1.2.3");
}

#[test]
fn package_shows_package_json_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), r#"{"version":"2.0.1"}"#).unwrap();
    let seg = BuiltinModule::Package
        .render(&project_ctx(dir.path()), &FakeProbes::default())
        .unwrap();
    assert_eq!(seg.text, "\u{f03d7} 2.0.1");
}

#[test]
fn package_hides_without_a_declared_version() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), r#"{"name":"x"}"#).unwrap();
    let seg = BuiltinModule::Package.render(&project_ctx(dir.path()), &FakeProbes::default());
    assert_eq!(seg, None);
}

#[test]
fn language_modules_hide_when_nothing_detected() {
    let dir = tempfile::tempdir().unwrap();
    for module in [
        BuiltinModule::Package,
        BuiltinModule::Unity,
        BuiltinModule::Node,
        BuiltinModule::Rust,
        BuiltinModule::Go,
        BuiltinModule::Python,
        BuiltinModule::Dotnet,
        BuiltinModule::Ruby,
        BuiltinModule::Java,
        BuiltinModule::Kotlin,
        BuiltinModule::Php,
        BuiltinModule::Swift,
    ] {
        let seg = module.render(&project_ctx(dir.path()), &FakeProbes::default());
        assert_eq!(seg, None, "{module:?} should hide in an empty directory");
    }
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

#[test]
fn git_conflict_renders_red_with_warning_icon() {
    let probes = git_probes()
        .with_cmd("git branch --show-current", "main\n")
        .with_cmd("git status --porcelain", "UU conflicted.rs\n");
    let seg = BuiltinModule::Git.render(&git_ctx(), &probes).unwrap();
    assert_eq!(seg.text, "\u{e0a0} main \u{26a0}");
    assert_eq!(seg.style, rgb(0xF9, 0x26, 0x72));
}

#[test]
fn git_shows_renamed_and_deleted_marks() {
    let probes = git_probes()
        .with_cmd("git branch --show-current", "main\n")
        .with_cmd(
            "git status --porcelain",
            "R  old.rs -> new.rs\n D gone.rs\n",
        );
    let seg = BuiltinModule::Git.render(&git_ctx(), &probes).unwrap();
    assert_eq!(seg.text, "\u{e0a0} main \u{00bb}\u{2718}");
    assert_eq!(seg.style, rgb(0xE6, 0xDB, 0x74));
}

#[test]
fn git_shows_stash_count_on_a_clean_tree() {
    let probes = git_probes()
        .with_cmd("git branch --show-current", "main\n")
        .with_cmd("git status --porcelain", "")
        .with_cmd(
            "git stash list",
            "stash@{0}: WIP on main\nstash@{1}: WIP on main\n",
        );
    let seg = BuiltinModule::Git.render(&git_ctx(), &probes).unwrap();
    assert_eq!(seg.text, "\u{e0a0} main \u{2713}\u{2691}2");
    assert_eq!(seg.style, rgb(0xA6, 0xE2, 0x2E));
}

//! Git branch and working-tree state, via the `git` CLI.
//!
//! `GIT_OPTIONAL_LOCKS=0` on every call so a statusline refresh can never
//! contend on `.git/index.lock`.

use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;
use std::path::Path;

const GIT_ENV: &[(&str, &str)] = &[("GIT_OPTIONAL_LOCKS", "0")];

pub fn render(context: &Context, probes: &dyn Probes) -> Option<Segment> {
    let cwd = context
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
        .or(context.cwd.as_deref())?;
    let cwd = Path::new(cwd);
    let git = |args: &[&str]| probes.run("git", args, GIT_ENV, Some(cwd));

    // Not a repository → hidden.
    let git_dir_raw = git(&["rev-parse", "--git-dir"])?;
    let git_dir = git_dir_raw.trim();
    let git_dir = if Path::new(git_dir).is_relative() {
        cwd.join(git_dir)
    } else {
        Path::new(git_dir).to_path_buf()
    };

    let branch = match git(&["branch", "--show-current"]) {
        Some(name) if !name.trim().is_empty() => format!("\u{e0a0} {}", name.trim()),
        _ => {
            let short = git(&["rev-parse", "--short", "HEAD"])
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            format!("\u{f0e2c} {short}")
        }
    };

    let merging = git_dir.join("MERGE_HEAD").is_file();
    let rebasing = git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply").is_dir();
    let mut icons = String::new();
    if merging {
        icons.push('\u{26a1}'); // ⚡
    } else if rebasing {
        icons.push('\u{27f3}'); // ⟳
    }

    let status = git(&["status", "--porcelain"]).unwrap_or_default();
    let staged = status
        .lines()
        .any(|l| l.chars().next().is_some_and(|c| "MADRCU".contains(c)));
    let modified = status.lines().any(|l| l.starts_with(" M"));
    let untracked = status.lines().any(|l| l.starts_with("??"));
    if status.trim().is_empty() {
        icons.push('\u{2713}'); // ✓
    } else {
        if staged {
            icons.push('\u{25cf}'); // ●
        }
        if modified {
            icons.push('\u{270e}'); // ✎
        }
        if untracked {
            icons.push('?');
        }
    }

    if let Some(upstream) = git(&["rev-parse", "--abbrev-ref", "@{u}"]) {
        let upstream = upstream.trim();
        if !upstream.is_empty()
            && let Some(counts) = git(&[
                "rev-list",
                "--left-right",
                "--count",
                &format!("HEAD...{upstream}"),
            ])
        {
            let mut parts = counts.split_whitespace();
            let ahead: u64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            let behind: u64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            if ahead > 0 {
                icons.push_str(&format!("\u{2191}{ahead}")); // ↑
            }
            if behind > 0 {
                icons.push_str(&format!("\u{2193}{behind}")); // ↓
            }
        }
    }

    let style = if merging || rebasing {
        theme::RED
    } else if staged || modified || untracked {
        theme::YELLOW
    } else {
        theme::GREEN
    };
    Some(Segment::styled(format!("{branch} {icons}"), style))
}

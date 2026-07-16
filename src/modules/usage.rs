//! Claude usage quotas.
//!
//! Default source: the official `rate_limits` object on stdin (5-hour and
//! 7-day windows). With `scoped = true`, per-model quotas (e.g. Fable) are
//! read from a stale-while-revalidate cache maintained by the
//! `--refresh-usage` background process, which talks to an unofficial
//! endpoint — data is served from disk here, never fetched inline.

use crate::config::{Config, UsageConfig, default_cache_dir};
use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::theme;
use std::process::Stdio;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Normal,
    Warning,
    Critical,
}

struct Window {
    label: String,
    percent: u32,
    severity: Severity,
}

pub fn render(context: &Context, config: &Config, _probes: &dyn Probes) -> Option<Segment> {
    let cfg = &config.usage;
    let windows = if cfg.scoped {
        scoped_windows(cfg, context.version.as_deref()).unwrap_or_else(|| stdin_windows(context))
    } else {
        stdin_windows(context)
    };
    let windows: Vec<Window> = windows
        .into_iter()
        .filter(|w| matches_only(&w.label, &cfg.only))
        .collect();
    if windows.is_empty() {
        return None;
    }

    let text = windows
        .iter()
        .map(window_text)
        .collect::<Vec<_>>()
        .join(" \u{b7} ");
    let style = match windows.iter().map(|w| w.severity).max().unwrap() {
        Severity::Critical => theme::RED,
        Severity::Warning => theme::ORANGE,
        Severity::Normal => theme::GRAY,
    };
    Some(Segment::styled(text, style))
}

fn window_text(window: &Window) -> String {
    let mark = match window.severity {
        Severity::Normal => "",
        Severity::Warning => "\u{26a0}",  // ⚠
        Severity::Critical => "\u{26d4}", // ⛔
    };
    format!("{} {}%{}", window.label, window.percent, mark)
}

fn severity_from_percent(percent: f64) -> Severity {
    if percent >= 95.0 {
        Severity::Critical
    } else if percent >= 80.0 {
        Severity::Warning
    } else {
        Severity::Normal
    }
}

fn stdin_windows(context: &Context) -> Vec<Window> {
    let Some(limits) = context.rate_limits.as_ref() else {
        return Vec::new();
    };
    let mut windows = Vec::new();
    let mut push = |label: &str, window: &Option<crate::context::RateWindow>| {
        if let Some(percent) = window.as_ref().and_then(|w| w.used_percentage) {
            windows.push(Window {
                label: label.to_string(),
                percent: percent.round() as u32,
                severity: severity_from_percent(percent),
            });
        }
    };
    push("5h", &limits.five_hour);
    push("wk", &limits.seven_day);
    windows
}

/// Reads the scoped-quota cache; spawns a background refresh when stale.
/// Returns `None` (→ stdin fallback) when the cache is missing or unusable.
fn scoped_windows(cfg: &UsageConfig, cc_version: Option<&str>) -> Option<Vec<Window>> {
    let dir = cfg.cache_dir.clone().or_else(default_cache_dir)?;
    let path = dir.join("usage.json");

    let ttl = Duration::from_secs(cfg.cache_ttl.unwrap_or(120));
    let fresh = std::fs::metadata(&path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < ttl);
    if !fresh && cfg.auto_refresh {
        spawn_refresh(cc_version);
    }

    // Stale-while-revalidate: serve whatever is on disk right now.
    let body = std::fs::read_to_string(&path).ok()?;
    let doc: serde_json::Value = serde_json::from_str(&body).ok()?;
    let limits = doc.get("limits")?.as_array()?;

    let mut windows = Vec::new();
    for limit in limits {
        let percent = limit.get("percent").and_then(|p| p.as_f64()).unwrap_or(0.0);
        let severity = match limit.get("severity").and_then(|s| s.as_str()) {
            Some("critical") => Severity::Critical,
            Some("warning") => Severity::Warning,
            _ => Severity::Normal,
        };
        let kind = limit.get("kind").and_then(|k| k.as_str()).unwrap_or("");
        let label = match kind {
            "session" => "5h".to_string(),
            "weekly_all" => "wk".to_string(),
            _ => {
                let is_scoped_weekly = limit.get("group").and_then(|g| g.as_str())
                    == Some("weekly")
                    && limit.get("scope").is_some_and(|s| !s.is_null());
                if !is_scoped_weekly {
                    continue;
                }
                limit
                    .pointer("/scope/model/display_name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("?")
                    .to_string()
            }
        };
        windows.push(Window {
            label,
            percent: percent.round() as u32,
            severity,
        });
    }
    if windows.is_empty() {
        None
    } else {
        Some(windows)
    }
}

fn matches_only(label: &str, only: &[String]) -> bool {
    if only.is_empty() {
        return true;
    }
    let label_lower = label.to_lowercase();
    only.iter().any(|entry| {
        let entry = entry.to_lowercase();
        match entry.as_str() {
            "five_hour" | "5h" | "session" => label_lower == "5h",
            "seven_day" | "wk" | "weekly" => label_lower == "wk",
            _ => label_lower.contains(&entry),
        }
    })
}

fn spawn_refresh(cc_version: Option<&str>) {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--refresh-usage")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(version) = cc_version {
        cmd.env("CCSTATUSKIT_CC_VERSION", version);
    }
    let _ = cmd.spawn();
}

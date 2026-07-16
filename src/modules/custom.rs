//! External module execution — the `[custom.x]` contract.
//!
//! The child receives the raw Claude Code JSON on stdin plus `CCSK_*`
//! convenience env vars. Its first stdout line becomes the segment; empty
//! output, a nonzero exit, or hitting the timeout hides the module. The
//! child is killed at the deadline so nothing outlives the statusline.

use crate::config::CustomModuleConfig;
use crate::context::Context;
use crate::segment::Segment;
use crate::style::parse_style;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn render(
    cfg: &CustomModuleConfig,
    context: &Context,
    palette: &HashMap<String, String>,
    timeout: Duration,
) -> Option<Segment> {
    if cfg.disabled || cfg.command.is_empty() {
        return None;
    }
    if !when_dir_matches(cfg, context) {
        return None;
    }

    let shell = cfg.shell.clone().unwrap_or_else(default_shell);
    let (program, shell_args) = shell.split_first()?;
    let mut command = Command::new(program);
    command
        .args(shell_args)
        .arg(&cfg.command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env("CCSK_CONTRACT", "1");
    for (key, value) in contract_env(context) {
        command.env(key, value);
    }

    let mut child = command.spawn().ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        // If the child exits without reading, the write fails — that's fine.
        let _ = stdin.write_all(context.raw.as_bytes());
    }

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                let mut output = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    let _ = stdout.read_to_string(&mut output);
                }
                let line = output.lines().next().unwrap_or("").trim_end();
                if line.is_empty() {
                    return None;
                }
                let style = cfg
                    .style
                    .as_deref()
                    .and_then(|spec| parse_style(spec, palette).ok())
                    .unwrap_or_default();
                return Some(Segment::styled(line.to_string(), style));
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(_) => return None,
        }
    }
}

fn when_dir_matches(cfg: &CustomModuleConfig, context: &Context) -> bool {
    if cfg.when_dir.is_empty() {
        return true;
    }
    let Some(base) = context
        .workspace
        .as_ref()
        .and_then(|w| w.project_dir.as_deref())
        .or(context.cwd.as_deref())
    else {
        return false;
    };
    let base = Path::new(base);
    cfg.when_dir.iter().any(|entry| base.join(entry).exists())
}

fn default_shell() -> Vec<String> {
    if cfg!(windows) {
        vec!["cmd".to_string(), "/C".to_string()]
    } else {
        vec!["sh".to_string(), "-c".to_string()]
    }
}

fn contract_env(context: &Context) -> Vec<(&'static str, String)> {
    let workspace = context.workspace.as_ref();
    let pairs = [
        (
            "CCSK_MODEL",
            context
                .model
                .as_ref()
                .and_then(|m| m.display_name.as_deref()),
        ),
        (
            "CCSK_MODEL_ID",
            context.model.as_ref().and_then(|m| m.id.as_deref()),
        ),
        ("CCSK_CWD", context.cwd.as_deref()),
        (
            "CCSK_CURRENT_DIR",
            workspace.and_then(|w| w.current_dir.as_deref()),
        ),
        (
            "CCSK_PROJECT_DIR",
            workspace.and_then(|w| w.project_dir.as_deref()),
        ),
        ("CCSK_SESSION_ID", context.session_id.as_deref()),
        ("CCSK_TRANSCRIPT_PATH", context.transcript_path.as_deref()),
        ("CCSK_VERSION", context.version.as_deref()),
    ];
    let mut env: Vec<(&'static str, String)> = pairs
        .into_iter()
        .filter_map(|(key, value)| value.map(|v| (key, v.to_string())))
        .collect();
    if let Some(pct) = context
        .context_window
        .as_ref()
        .and_then(|cw| cw.used_percentage)
    {
        env.push(("CCSK_CTX_PCT", format!("{}", pct as u32)));
    }
    env
}

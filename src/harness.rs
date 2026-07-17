//! `ccstatuskit test-module <command>` — exercises a command against the
//! module contract so authors can see exactly what a render would do.
//!
//! Unlike the render path, stderr is captured and shown, timing is compared
//! against the default budget, and the process runs under a generous cap so
//! slow modules are diagnosed instead of silently hidden.

use crate::context::Context;
use crate::modules::custom::{contract_env, default_shell, kill_tree};
use std::io::{Read, Write as _};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const FULL_FIXTURE: &str = include_str!("../tests/fixtures/full.json");
const HARNESS_CAP: Duration = Duration::from_secs(2);
const RENDER_BUDGET_MS: u128 = 250;

pub fn run(command: &str) -> i32 {
    println!("Testing module command: {command}");
    println!("Contract: docs/module-contract.md (CCSK_CONTRACT=1)");
    let mut failures = 0;

    for (label, json) in [("full context", FULL_FIXTURE), ("empty context", "{}")] {
        println!("\nScenario: {label}");
        match run_scenario(command, json) {
            Err(message) => {
                println!("  \u{2717} {message}");
                failures += 1;
            }
            Ok(outcome) => outcome.print(),
        }
    }

    println!();
    if failures > 0 {
        println!("Result: FAIL ({failures} scenario(s) failed)");
        1
    } else {
        println!("Result: OK — behavior above is what a render would show");
        0
    }
}

struct Outcome {
    elapsed_ms: u128,
    exit_ok: bool,
    exit_code: Option<i32>,
    first_line: String,
    extra_lines: usize,
    stderr: String,
}

impl Outcome {
    fn print(&self) {
        println!(
            "  time: {}ms (render budget {}ms)",
            self.elapsed_ms, RENDER_BUDGET_MS
        );
        if self.elapsed_ms > RENDER_BUDGET_MS {
            println!("  \u{26a0} over the default budget — a render would drop this segment");
        }
        if !self.exit_ok {
            println!(
                "  segment: hidden (exit code {:?} — nonzero exits hide the module)",
                self.exit_code
            );
        } else if self.first_line.is_empty() {
            println!("  segment: hidden (empty output — the idiomatic way to be conditional)");
        } else {
            println!("  segment: {:?}", self.first_line);
            if self.extra_lines > 0 {
                println!(
                    "  \u{26a0} {} extra output line(s) are ignored by the render",
                    self.extra_lines
                );
            }
        }
        if !self.stderr.trim().is_empty() {
            println!("  stderr: {}", self.stderr.trim_end());
        }
    }
}

fn run_scenario(command: &str, json: &str) -> Result<Outcome, String> {
    let context = Context::from_stdin_json(json.to_string());
    let shell = default_shell();
    let (program, shell_args) = shell
        .split_first()
        .ok_or_else(|| "empty shell".to_string())?;

    let mut child = Command::new(program);
    child
        .args(shell_args)
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("CCSK_CONTRACT", "1");
    for (key, value) in contract_env(&context) {
        child.env(key, value);
    }
    let mut child = child
        .spawn()
        .map_err(|err| format!("failed to spawn `{program}`: {err}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(context.raw.as_bytes());
    }

    let started = Instant::now();
    let deadline = started + HARNESS_CAP;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let elapsed_ms = started.elapsed().as_millis();
                let mut stdout = String::new();
                if let Some(mut pipe) = child.stdout.take() {
                    let _ = pipe.read_to_string(&mut stdout);
                }
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stderr.take() {
                    let _ = pipe.read_to_string(&mut stderr);
                }
                let mut lines = stdout.lines();
                let first_line = lines.next().unwrap_or("").trim_end().to_string();
                return Ok(Outcome {
                    elapsed_ms,
                    exit_ok: status.success(),
                    exit_code: status.code(),
                    first_line,
                    extra_lines: lines.filter(|l| !l.trim().is_empty()).count(),
                    stderr,
                });
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    kill_tree(&mut child);
                    return Err(format!(
                        "timeout: no exit within {}s (a render would kill it at {}ms)",
                        HARNESS_CAP.as_secs(),
                        RENDER_BUDGET_MS
                    ));
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(err) => return Err(format!("wait failed: {err}")),
        }
    }
}

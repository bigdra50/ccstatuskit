//! Parallel module execution and fail-soft row assembly.
//!
//! Every unique module referenced by the template runs on its own detached
//! thread. Results are collected until the global budget expires; whatever
//! has not arrived by then is treated as hidden. A panicking module drops
//! its channel sender, which reads as hidden too — a broken module can cost
//! its segment, never the statusline.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::format::{ModuleRef, Row, Token};
use crate::segment::Segment;

pub trait ModuleRunner: Send + Sync {
    fn run(&self, module: &ModuleRef) -> Option<Segment>;
}

pub struct EngineOptions {
    /// Global wall-clock budget for all modules together.
    pub budget: Duration,
    /// Emit ANSI styling. Disable for NO_COLOR.
    pub color: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        EngineOptions {
            budget: Duration::from_millis(500),
            color: true,
        }
    }
}

pub fn render_rows(
    rows: &[Row],
    runner: Arc<dyn ModuleRunner>,
    opts: &EngineOptions,
) -> Vec<String> {
    let results = run_modules(rows, runner, opts.budget);
    assemble(rows, &results, opts.color)
}

fn unique_modules(rows: &[Row]) -> Vec<ModuleRef> {
    let mut seen = Vec::new();
    for token in rows.iter().flatten() {
        if let Token::Module(m) = token
            && !seen.contains(m)
        {
            seen.push(m.clone());
        }
    }
    seen
}

fn run_modules(
    rows: &[Row],
    runner: Arc<dyn ModuleRunner>,
    budget: Duration,
) -> HashMap<ModuleRef, Segment> {
    let modules = unique_modules(rows);
    let (tx, rx) = mpsc::channel();
    for module in modules.iter().cloned() {
        let tx = tx.clone();
        let runner = Arc::clone(&runner);
        std::thread::spawn(move || {
            // A panic inside run() drops `tx` without sending: fail-soft.
            let result = runner.run(&module);
            let _ = tx.send((module, result));
        });
    }
    drop(tx);

    let deadline = Instant::now() + budget;
    let mut results = HashMap::new();
    let mut pending = modules.len();
    while pending > 0 {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(remaining) {
            Ok((module, Some(segment))) => {
                results.insert(module, segment);
                pending -= 1;
            }
            Ok((_, None)) => pending -= 1,
            // Timeout: abandon stragglers. Disconnected: every worker is done
            // or has panicked.
            Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    results
}

fn assemble(rows: &[Row], results: &HashMap<ModuleRef, Segment>, color: bool) -> Vec<String> {
    let mut lines = Vec::new();
    for row in rows {
        let mut out = String::new();
        let mut has_segment = false;
        let mut has_literal_ink = false;
        for token in row {
            match token {
                Token::Text(text) => {
                    out.push_str(text);
                    if !text.trim().is_empty() {
                        has_literal_ink = true;
                    }
                }
                Token::Module(module) => {
                    if let Some(segment) = results.get(module) {
                        segment.render_into(&mut out, color);
                        has_segment = true;
                    }
                }
            }
        }
        if has_segment || has_literal_ink {
            lines.push(out.trim().to_string());
        }
    }
    lines
}

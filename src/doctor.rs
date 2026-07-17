//! `ccstatuskit doctor` — human-facing diagnosis of the user's setup.
//!
//! The render path deliberately swallows configuration mistakes (fail-soft);
//! this is where they become visible. Errors are mistakes the user should
//! fix (broken TOML, bad styles, empty commands); warnings are things that
//! silently hide at render time (unknown module names, missing tables).

use crate::config::{Config, default_cache_dir, resolved_config_path};
use crate::format::{ModuleRef, Token, parse_format};
use crate::modules::BuiltinModule;
use crate::modules::custom::default_shell;
use crate::style::parse_style;
use std::fmt::Write as _;
use std::path::Path;

pub struct Report {
    pub text: String,
    pub errors: usize,
    pub warnings: usize,
}

impl Report {
    fn ok(&mut self, msg: &str) {
        let _ = writeln!(self.text, "\u{2713} {msg}");
    }
    fn warn(&mut self, msg: &str) {
        self.warnings += 1;
        let _ = writeln!(self.text, "\u{26a0} {msg}");
    }
    fn error(&mut self, msg: &str) {
        self.errors += 1;
        let _ = writeln!(self.text, "\u{2717} {msg}");
    }
}

pub fn run() -> Report {
    let mut report = Report {
        text: String::new(),
        errors: 0,
        warnings: 0,
    };

    let config = check_config_file(&mut report);
    check_format(&mut report, &config);
    check_styles(&mut report, &config);
    check_custom(&mut report, &config);
    check_usage(&mut report, &config);

    let _ = writeln!(
        report.text,
        "\n{} error(s), {} warning(s)",
        report.errors, report.warnings
    );
    report
}

fn check_config_file(report: &mut Report) -> Config {
    let Some(path) = resolved_config_path() else {
        report.warn("no config path could be resolved (no home directory?); using defaults");
        return Config::default();
    };
    if !path.is_file() {
        report.ok(&format!(
            "config: {} (not present — using built-in defaults)",
            path.display()
        ));
        return Config::default();
    }
    let body = match std::fs::read_to_string(&path) {
        Ok(body) => body,
        Err(err) => {
            report.error(&format!("config {} is unreadable: {err}", path.display()));
            return Config::default();
        }
    };
    match toml::from_str::<Config>(&body) {
        Ok(config) => {
            report.ok(&format!("config: {}", path.display()));
            config
        }
        Err(err) => {
            report.error(&format!(
                "config {}: TOML parse error — the render falls back to defaults:\n  {err}",
                path.display()
            ));
            Config::default()
        }
    }
}

fn check_format(report: &mut Report, config: &Config) {
    let rows = match parse_format(config.format()) {
        Ok(rows) => rows,
        Err(err) => {
            report.error(&format!("format template: {err}"));
            return;
        }
    };
    report.ok(&format!("format: {} row(s)", rows.len()));
    for token in rows.iter().flatten() {
        let Token::Module(module) = token else {
            continue;
        };
        match module {
            ModuleRef::Builtin(name) => {
                if BuiltinModule::from_name(name).is_none() {
                    report.warn(&format!("unknown module `${name}` — it renders as hidden"));
                }
            }
            ModuleRef::Custom(name) => {
                if !config.custom.contains_key(name) {
                    report.warn(&format!(
                        "`${{custom.{name}}}` has no [custom.{name}] table — it renders as hidden"
                    ));
                }
            }
        }
    }
}

fn check_styles(report: &mut Report, config: &Config) {
    for module in BuiltinModule::ALL {
        let (_, style) = config.module_basics(module);
        if let Some(spec) = style
            && let Err(err) = parse_style(spec, &config.palette)
        {
            report.error(&format!("[{}] style: {err}", module.name()));
        }
    }
    for (name, value) in &config.palette {
        if parse_style(&format!("fg:{value}"), &Default::default()).is_err() {
            report.error(&format!(
                "[palette] {name} = \"{value}\" is not a valid color"
            ));
        }
    }
}

fn check_custom(report: &mut Report, config: &Config) {
    for (name, custom) in &config.custom {
        if custom.command.trim().is_empty() {
            report.error(&format!("[custom.{name}] command is empty"));
        }
        if let Some(spec) = &custom.style
            && let Err(err) = parse_style(spec, &config.palette)
        {
            report.error(&format!("[custom.{name}] style: {err}"));
        }
        let shell = custom.shell.clone().unwrap_or_else(default_shell);
        match shell.first() {
            None => report.error(&format!("[custom.{name}] shell is an empty list")),
            Some(program) => {
                if !on_path(program) {
                    report.warn(&format!(
                        "[custom.{name}] shell `{program}` not found on PATH"
                    ));
                }
            }
        }
    }
}

fn check_usage(report: &mut Report, config: &Config) {
    if !config.usage.scoped {
        return;
    }
    if crate::auth::read_token().is_some() {
        report.ok("scoped usage: OAuth credentials found");
    } else {
        report.warn(
            "scoped usage is enabled but no OAuth credentials were found — \
             per-model windows will fall back to stdin data",
        );
    }
    let Some(dir) = config.usage.cache_dir.clone().or_else(default_cache_dir) else {
        report.warn("scoped usage: no cache directory could be resolved");
        return;
    };
    let cache = dir.join("usage.json");
    if let Ok(meta) = std::fs::metadata(&cache) {
        let age = meta
            .modified()
            .ok()
            .and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
            .map(|d| d.as_secs());
        match age {
            Some(secs) => report.ok(&format!(
                "scoped usage cache: {} ({secs}s old)",
                cache.display()
            )),
            None => report.ok(&format!("scoped usage cache: {}", cache.display())),
        }
    } else {
        report.ok("scoped usage cache: not yet populated (the first render refreshes it)");
    }
}

fn on_path(program: &str) -> bool {
    if program.contains('/') || program.contains('\\') {
        return Path::new(program).exists();
    }
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| {
        if dir.join(program).is_file() {
            return true;
        }
        if cfg!(windows) {
            for ext in ["exe", "bat", "cmd"] {
                if dir.join(format!("{program}.{ext}")).is_file() {
                    return true;
                }
            }
        }
        false
    })
}

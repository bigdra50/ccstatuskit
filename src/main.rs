use ccstatuskit::config::{Config, DEFAULT_FORMAT};
use ccstatuskit::context::Context;
use ccstatuskit::engine::{EngineOptions, render_rows};
use ccstatuskit::format::parse_format;
use ccstatuskit::probes::RealProbes;
use ccstatuskit::runner::BuiltinRunner;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    // A statusline must never break Claude Code's UI: consume stdin,
    // render what we can, and always exit 0.
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    let context = Context::from_stdin_json(input);
    let config = Config::load();

    let rows = parse_format(config.format()).unwrap_or_else(|err| {
        eprintln!("ccstatuskit: {err}; falling back to the default format");
        parse_format(DEFAULT_FORMAT).expect("default format must parse")
    });

    let options = EngineOptions {
        budget: Duration::from_millis(config.command_timeout.unwrap_or(250)),
        color: std::env::var_os("NO_COLOR").is_none(),
    };
    let runner = Arc::new(BuiltinRunner {
        context,
        config,
        probes: Box::new(RealProbes),
    });

    // Write \n only — bypass any platform newline translation.
    let mut stdout = std::io::stdout().lock();
    for line in render_rows(&rows, runner, &options) {
        let _ = stdout.write_all(line.as_bytes());
        let _ = stdout.write_all(b"\n");
    }
}

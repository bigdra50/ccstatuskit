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
    // On Windows, the std handles we inherited (Claude Code's statusline
    // pipe) are themselves inheritable; any child we spawn — and its
    // descendants — would receive them and could hold the pipe open long
    // after we exit, stalling the statusline. Clear the inherit flag first.
    #[cfg(windows)]
    make_std_handles_uninheritable();

    if std::env::args().nth(1).as_deref() == Some("--refresh-usage") {
        ccstatuskit::refresh::run();
        return;
    }

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

#[cfg(windows)]
fn make_std_handles_uninheritable() {
    use windows_sys::Win32::Foundation::{
        HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, SetHandleInformation,
    };
    use windows_sys::Win32::System::Console::{
        GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };
    for kind in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // SAFETY: GetStdHandle/SetHandleInformation on our own std handles.
        unsafe {
            let handle = GetStdHandle(kind);
            if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
                SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0);
            }
        }
    }
}

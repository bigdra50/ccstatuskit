use assert_cmd::Command;

fn test_module(command: &str) -> (String, bool) {
    let output = Command::cargo_bin("ccstatuskit")
        .unwrap()
        .args(["test-module", command])
        .output()
        .unwrap();
    (
        String::from_utf8(output.stdout).unwrap(),
        output.status.success(),
    )
}

#[test]
fn emitting_command_reports_its_segment() {
    // `echo` behaves the same under sh and cmd.
    let (out, ok) = test_module("echo hi-from-module");
    assert!(ok, "got: {out}");
    assert!(out.contains("hi-from-module"), "got: {out}");
    assert!(out.contains("full context"), "got: {out}");
    assert!(out.contains("empty context"), "got: {out}");
}

#[test]
fn silent_command_reports_hidden() {
    // `cd .` outputs nothing under both sh and cmd.
    let (out, ok) = test_module("cd .");
    assert!(ok, "hidden is a valid outcome: {out}");
    assert!(out.to_lowercase().contains("hidden"), "got: {out}");
}

#[test]
fn missing_command_argument_fails_with_usage() {
    let output = Command::cargo_bin("ccstatuskit")
        .unwrap()
        .arg("test-module")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.to_lowercase().contains("usage"), "got: {err}");
}

#[cfg(unix)]
#[test]
fn slow_command_reports_timeout_and_fails() {
    let (out, ok) = test_module("sleep 30; echo late");
    assert!(
        !ok,
        "a module that overruns the harness cap must fail: {out}"
    );
    assert!(out.to_lowercase().contains("timeout"), "got: {out}");
}

#[cfg(unix)]
#[test]
fn contract_env_reaches_the_module() {
    let (out, ok) = test_module("printf 'model=%s' \"$CCSK_MODEL\"");
    assert!(ok, "got: {out}");
    assert!(out.contains("model=Opus"), "got: {out}");
}

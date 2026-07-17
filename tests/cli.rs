use assert_cmd::Command;

/// The core contract: no matter what arrives on stdin, the binary exits 0.
/// (With an empty context, always-on modules like `time` may still render.)
#[test]
fn binary_exits_zero_on_empty_stdin() {
    let mut cmd = Command::cargo_bin("ccstatuskit").unwrap();
    cmd.env("CCSTATUSKIT_CONFIG", "/nonexistent/config.toml")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn version_flag_prints_version_without_reading_stdin() {
    let assert = Command::cargo_bin("ccstatuskit")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(out.contains(env!("CARGO_PKG_VERSION")), "got: {out:?}");
    assert!(!out.contains('\u{1b}'), "should not render a statusline");
}

#[test]
fn help_flag_prints_usage() {
    let assert = Command::cargo_bin("ccstatuskit")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(out.contains("statusline"), "got: {out:?}");
    assert!(out.contains("--version"));
}

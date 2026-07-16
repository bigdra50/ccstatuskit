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

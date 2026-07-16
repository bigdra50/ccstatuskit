use assert_cmd::Command;

#[test]
fn binary_exits_zero_on_empty_stdin() {
    let mut cmd = Command::cargo_bin("ccstatuskit").unwrap();
    cmd.write_stdin("").assert().success().stdout("");
}

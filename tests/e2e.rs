use assert_cmd::Command;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn run(config_body: &str, stdin: &str, color: bool) -> String {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(&config, config_body).unwrap();
    let mut cmd = Command::cargo_bin("ccstatuskit").unwrap();
    cmd.env("CCSTATUSKIT_CONFIG", &config);
    if !color {
        cmd.env("NO_COLOR", "1");
    } else {
        cmd.env_remove("NO_COLOR");
    }
    let assert = cmd.write_stdin(stdin.to_string()).assert().success();
    String::from_utf8(assert.get_output().stdout.clone()).unwrap()
}

#[test]
fn renders_configured_modules_from_stdin() {
    let out = run("format = \"$model | $ctx\"\n", &fixture("full"), false);
    assert_eq!(out.trim_end(), "\u{f06a9} Opus | CTX 8%");
}

#[test]
fn multi_line_format_produces_multiple_rows() {
    let out = run("format = \"$model\\n$ctx\"\n", &fixture("full"), false);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["\u{f06a9} Opus", "CTX 8%"]);
}

#[test]
fn malformed_stdin_still_exits_zero() {
    let out = run("format = \"$model $ctx\"\n", "garbage {", false);
    assert_eq!(out.trim_end(), "");
}

#[test]
fn disabled_module_is_hidden() {
    let out = run(
        "format = \"$model $ctx\"\n[ctx]\ndisabled = true\n",
        &fixture("full"),
        false,
    );
    assert_eq!(out.trim_end(), "\u{f06a9} Opus");
}

#[test]
fn style_override_uses_palette() {
    let out = run(
        concat!(
            "format = \"$ctx\"\n",
            "[palette]\nhot = \"#FF0000\"\n",
            "[ctx]\nstyle = \"bold fg:hot\"\n",
        ),
        &fixture("full"),
        true,
    );
    assert!(out.contains("\u{1b}[1m"), "missing bold: {out:?}");
    assert!(out.contains("38;2;255;0;0"), "missing rgb: {out:?}");
}

#[test]
fn broken_config_falls_back_to_defaults_and_exits_zero() {
    let out = run("format = [this is not toml", &fixture("minimal"), false);
    // Default template renders at least the model segment.
    assert!(out.contains("Opus"), "got: {out:?}");
}

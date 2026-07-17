use assert_cmd::Command;

fn doctor_with(config_body: &str) -> (String, bool) {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(&config, config_body).unwrap();
    let output = Command::cargo_bin("ccstatuskit")
        .unwrap()
        .env("CCSTATUSKIT_CONFIG", &config)
        .arg("doctor")
        .output()
        .unwrap();
    (
        String::from_utf8(output.stdout).unwrap(),
        output.status.success(),
    )
}

#[test]
fn valid_config_passes_with_summary() {
    let (out, ok) = doctor_with("format = \"$model $ctx\"\n");
    assert!(ok, "expected exit 0, got: {out}");
    assert!(out.contains("0 error"), "got: {out}");
}

#[test]
fn broken_toml_is_an_error() {
    let (out, ok) = doctor_with("format = [not toml");
    assert!(!ok, "expected exit 1, got: {out}");
    assert!(out.to_lowercase().contains("toml"), "got: {out}");
}

#[test]
fn unknown_module_name_is_a_warning_not_an_error() {
    let (out, ok) = doctor_with("format = \"$model $nonexistent\"\n");
    assert!(ok, "warnings must not fail the check: {out}");
    assert!(out.contains("unknown module"), "got: {out}");
    assert!(out.contains("nonexistent"), "got: {out}");
}

#[test]
fn unconfigured_custom_reference_is_a_warning() {
    let (out, ok) = doctor_with("format = \"${custom.ghost}\"\n");
    assert!(ok, "got: {out}");
    assert!(out.contains("ghost"), "got: {out}");
    assert!(out.contains("[custom.ghost]"), "got: {out}");
}

#[test]
fn invalid_style_is_an_error() {
    let (out, ok) = doctor_with("format = \"$model\"\n[model]\nstyle = \"fg:notacolor\"\n");
    assert!(!ok, "expected exit 1, got: {out}");
    assert!(out.contains("style"), "got: {out}");
    assert!(out.contains("notacolor"), "got: {out}");
}

#[test]
fn broken_format_template_is_an_error() {
    let (out, ok) = doctor_with("format = \"${model\"\n");
    assert!(!ok, "expected exit 1, got: {out}");
    assert!(out.contains("format"), "got: {out}");
}

#[test]
fn empty_custom_command_is_an_error() {
    let (out, ok) = doctor_with("format = \"${custom.x}\"\n[custom.x]\ncommand = \"\"\n");
    assert!(!ok, "expected exit 1, got: {out}");
    assert!(out.contains("command"), "got: {out}");
}

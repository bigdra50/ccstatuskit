//! Contract tests for `[custom.x]` external modules. These spawn real `sh`
//! processes; the config pins `shell = ["sh", "-c"]` so they behave the same
//! on GitHub's Windows runners (Git Bash provides `sh`).

use assert_cmd::Command;
use std::time::Instant;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn run_with_config(config_body: &str, stdin: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(&config, config_body).unwrap();
    let assert = Command::cargo_bin("ccstatuskit")
        .unwrap()
        .env("CCSTATUSKIT_CONFIG", &config)
        .env("NO_COLOR", "1")
        .write_stdin(stdin.to_string())
        .assert()
        .success();
    String::from_utf8(assert.get_output().stdout.clone()).unwrap()
}

#[test]
fn custom_module_receives_contract_env_and_raw_stdin() {
    let out = run_with_config(
        concat!(
            "format = \"${custom.probe}\"\n",
            "[custom.probe]\n",
            "shell = [\"sh\", \"-c\"]\n",
            "command = \"printf 'M=%s P=%s C=%s J=' \\\"$CCSK_MODEL\\\" \\\"$CCSK_PROJECT_DIR\\\" \\\"$CCSK_CTX_PCT\\\"; head -c 1\"\n",
        ),
        &fixture("full"),
    );
    assert_eq!(out.trim_end(), "M=Opus P=/workspace/project C=8 J={");
}

#[test]
fn custom_module_sets_contract_version() {
    let out = run_with_config(
        concat!(
            "format = \"${custom.v}\"\n",
            "[custom.v]\n",
            "shell = [\"sh\", \"-c\"]\n",
            "command = \"echo contract-$CCSK_CONTRACT\"\n",
        ),
        &fixture("minimal"),
    );
    assert_eq!(out.trim_end(), "contract-1");
}

#[test]
fn empty_output_hides_the_module() {
    let out = run_with_config(
        concat!(
            "format = \"left ${custom.quiet}\"\n",
            "[custom.quiet]\n",
            "shell = [\"sh\", \"-c\"]\n",
            "command = \"true\"\n",
        ),
        &fixture("minimal"),
    );
    assert_eq!(out.trim_end(), "left");
}

#[test]
fn nonzero_exit_hides_the_module_even_with_output() {
    let out = run_with_config(
        concat!(
            "format = \"left ${custom.broken}\"\n",
            "[custom.broken]\n",
            "shell = [\"sh\", \"-c\"]\n",
            "command = \"echo boom; exit 3\"\n",
        ),
        &fixture("minimal"),
    );
    assert_eq!(out.trim_end(), "left");
}

#[test]
fn only_the_first_output_line_is_used() {
    let out = run_with_config(
        concat!(
            "format = \"${custom.multi}\"\n",
            "[custom.multi]\n",
            "shell = [\"sh\", \"-c\"]\n",
            "command = \"printf 'one\\\\ntwo\\\\n'\"\n",
        ),
        &fixture("minimal"),
    );
    assert_eq!(out.trim_end(), "one");
}

#[test]
fn slow_command_is_killed_at_the_timeout() {
    let started = Instant::now();
    let out = run_with_config(
        concat!(
            "format = \"left ${custom.slow}\"\n",
            "command_timeout = 100\n",
            "[custom.slow]\n",
            "shell = [\"sh\", \"-c\"]\n",
            "command = \"sleep 5; echo late\"\n",
        ),
        &fixture("minimal"),
    );
    assert!(started.elapsed().as_millis() < 3000);
    assert_eq!(out.trim_end(), "left");
}

#[test]
fn config_style_wraps_the_output() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(
        &config,
        concat!(
            "format = \"${custom.styled}\"\n",
            "[custom.styled]\n",
            "shell = [\"sh\", \"-c\"]\n",
            "command = \"echo hi\"\n",
            "style = \"fg:red\"\n",
        ),
    )
    .unwrap();
    let assert = Command::cargo_bin("ccstatuskit")
        .unwrap()
        .env("CCSTATUSKIT_CONFIG", &config)
        .env_remove("NO_COLOR")
        .write_stdin(fixture("minimal"))
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(out.contains("\u{1b}[31m"), "missing red: {out:?}");
    assert!(out.contains("hi"));
}

#[test]
fn when_dir_gates_execution_on_project_contents() {
    let project = tempfile::tempdir().unwrap();
    let stdin = format!(
        r#"{{"workspace":{{"project_dir":{}}}}}"#,
        serde_json::to_string(project.path().to_str().unwrap()).unwrap()
    );
    let config_body = concat!(
        "format = \"left ${custom.unity}\"\n",
        "[custom.unity]\n",
        "shell = [\"sh\", \"-c\"]\n",
        "command = \"echo unity-ok\"\n",
        "when_dir = [\"Assets\"]\n",
    );

    let out = run_with_config(config_body, &stdin);
    assert_eq!(out.trim_end(), "left", "should hide without Assets/");

    std::fs::create_dir(project.path().join("Assets")).unwrap();
    let out = run_with_config(config_body, &stdin);
    assert_eq!(out.trim_end(), "left unity-ok");
}

use ccstatuskit::context::Context;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[test]
fn minimal_fixture_extracts_model_display_name() {
    let ctx = Context::from_stdin_json(fixture("minimal"));
    assert_eq!(ctx.model.unwrap().display_name.as_deref(), Some("Opus"));
    assert_eq!(ctx.cwd.as_deref(), Some("/home/user/project"));
}

#[test]
fn full_fixture_parses_all_key_fields() {
    let ctx = Context::from_stdin_json(fixture("full"));
    let ws = ctx.workspace.unwrap();
    assert_eq!(ws.project_dir.as_deref(), Some("/home/user/project"));
    assert_eq!(ws.repo.unwrap().name.as_deref(), Some("ccstatuskit"));
    let cw = ctx.context_window.unwrap();
    assert_eq!(cw.used_percentage, Some(8.25));
    let rl = ctx.rate_limits.unwrap();
    assert_eq!(rl.five_hour.unwrap().used_percentage, Some(23.5));
    assert_eq!(rl.seven_day.unwrap().resets_at, Some(1738857600));
    assert_eq!(ctx.cost.unwrap().total_duration_ms, Some(45000));
    assert_eq!(ctx.vim.unwrap().mode.as_deref(), Some("NORMAL"));
}

#[test]
fn unknown_fields_are_ignored() {
    let ctx = Context::from_stdin_json(fixture("extra_fields"));
    assert_eq!(ctx.model.unwrap().display_name.as_deref(), Some("Opus"));
    assert_eq!(ctx.context_window.unwrap().total_input_tokens, Some(100));
}

#[test]
fn drifted_field_types_do_not_poison_the_rest() {
    let ctx = Context::from_stdin_json(fixture("partial"));
    // Drifted fields degrade to None...
    assert!(ctx.context_window.is_none());
    assert!(ctx.cost.is_none());
    // ...while intact fields still parse.
    assert_eq!(ctx.cwd.as_deref(), Some("/home/user/project"));
    assert_eq!(
        ctx.rate_limits.unwrap().five_hour.unwrap().used_percentage,
        Some(23.5)
    );
}

#[test]
fn malformed_json_yields_default_context_with_raw_preserved() {
    let ctx = Context::from_stdin_json("not json at all {".to_string());
    assert!(ctx.cwd.is_none());
    assert!(ctx.model.is_none());
    assert_eq!(ctx.raw, "not json at all {");
}

#[test]
fn raw_input_is_preserved_verbatim_for_passthrough() {
    let raw = fixture("full");
    let ctx = Context::from_stdin_json(raw.clone());
    assert_eq!(ctx.raw, raw);
}

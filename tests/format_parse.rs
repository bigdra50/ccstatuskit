use ccstatuskit::format::{FormatError, ModuleRef, Token, parse_format};

fn builtin(name: &str) -> Token {
    Token::Module(ModuleRef::Builtin(name.to_string()))
}

fn custom(name: &str) -> Token {
    Token::Module(ModuleRef::Custom(name.to_string()))
}

fn text(s: &str) -> Token {
    Token::Text(s.to_string())
}

#[test]
fn splits_rows_on_newline() {
    let rows = parse_format("$model $git\n$usage").unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec![builtin("model"), text(" "), builtin("git")]);
    assert_eq!(rows[1], vec![builtin("usage")]);
}

#[test]
fn accepts_bare_and_braced_module_tokens() {
    let rows = parse_format("$model ${directory}").unwrap();
    assert_eq!(
        rows[0],
        vec![builtin("model"), text(" "), builtin("directory")]
    );
}

#[test]
fn custom_modules_use_braced_dotted_form() {
    let rows = parse_format("${custom.unilyze}").unwrap();
    assert_eq!(rows[0], vec![custom("unilyze")]);
}

#[test]
fn literal_text_around_modules_is_preserved() {
    let rows = parse_format("[$model]").unwrap();
    assert_eq!(rows[0], vec![text("["), builtin("model"), text("]")]);
}

#[test]
fn escaped_dollar_is_literal() {
    let rows = parse_format("\\$5 spent").unwrap();
    assert_eq!(rows[0], vec![text("$5 spent")]);
}

#[test]
fn escaped_backslash_is_literal() {
    let rows = parse_format("a\\\\b").unwrap();
    assert_eq!(rows[0], vec![text("a\\b")]);
}

#[test]
fn bare_dollar_before_non_identifier_is_literal() {
    let rows = parse_format("100$ left").unwrap();
    assert_eq!(rows[0], vec![text("100$ left")]);
}

#[test]
fn unbraced_name_stops_at_dot() {
    let rows = parse_format("$custom.x").unwrap();
    assert_eq!(rows[0], vec![builtin("custom"), text(".x")]);
}

#[test]
fn empty_rows_are_dropped() {
    let rows = parse_format("\n$model\n\n").unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], vec![builtin("model")]);
}

#[test]
fn unclosed_brace_is_an_error() {
    assert_eq!(parse_format("${model"), Err(FormatError::UnclosedBrace));
}

#[test]
fn empty_braced_name_is_an_error() {
    assert_eq!(parse_format("${}"), Err(FormatError::EmptyModuleName));
    assert_eq!(
        parse_format("${custom.}"),
        Err(FormatError::EmptyModuleName)
    );
}

#[test]
fn unknown_namespace_in_braces_is_an_error() {
    assert_eq!(
        parse_format("${plugin.foo}"),
        Err(FormatError::UnknownNamespace("plugin".to_string()))
    );
}

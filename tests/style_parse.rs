use anstyle::{AnsiColor, Color, RgbColor, Style};
use ccstatuskit::style::{StyleError, parse_style};
use std::collections::HashMap;

fn no_palette() -> HashMap<String, String> {
    HashMap::new()
}

#[test]
fn parses_bold_with_hex_foreground() {
    let style = parse_style("bold fg:#A6E22E", &no_palette()).unwrap();
    let expected = Style::new()
        .bold()
        .fg_color(Some(Color::Rgb(RgbColor(0xA6, 0xE2, 0x2E))));
    assert_eq!(style, expected);
}

#[test]
fn parses_named_ansi_color() {
    let style = parse_style("fg:green", &no_palette()).unwrap();
    assert_eq!(
        style,
        Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)))
    );
}

#[test]
fn parses_256_color_index_and_background() {
    let style = parse_style("fg:208 bg:236", &no_palette()).unwrap();
    let expected = Style::new()
        .fg_color(Some(Color::Ansi256(208.into())))
        .bg_color(Some(Color::Ansi256(236.into())));
    assert_eq!(style, expected);
}

#[test]
fn parses_all_attribute_tokens() {
    let style = parse_style("bold dimmed italic underline inverted", &no_palette()).unwrap();
    let expected = Style::new().bold().dimmed().italic().underline().invert();
    assert_eq!(style, expected);
}

#[test]
fn resolves_palette_names_before_parsing() {
    let palette = HashMap::from([("monokai_green".to_string(), "#A6E22E".to_string())]);
    let via_palette = parse_style("fg:monokai_green", &palette).unwrap();
    let direct = parse_style("fg:#A6E22E", &no_palette()).unwrap();
    assert_eq!(via_palette, direct);
}

#[test]
fn empty_style_is_a_plain_style() {
    let style = parse_style("", &no_palette()).unwrap();
    assert_eq!(style, Style::new());
}

#[test]
fn rendered_style_emits_ansi_and_reset() {
    let style = parse_style("bold", &no_palette()).unwrap();
    let rendered = format!("{style}text{style:#}");
    assert!(rendered.starts_with('\u{1b}'));
    assert!(rendered.contains("text"));
    assert!(rendered.ends_with('m'));
}

#[test]
fn unknown_token_is_an_error() {
    assert!(matches!(
        parse_style("blink", &no_palette()),
        Err(StyleError::UnknownToken(t)) if t == "blink"
    ));
}

#[test]
fn unknown_color_is_an_error() {
    assert!(matches!(
        parse_style("fg:definitely_not_a_color", &no_palette()),
        Err(StyleError::UnknownColor(c)) if c == "definitely_not_a_color"
    ));
}

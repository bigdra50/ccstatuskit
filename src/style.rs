//! Starship-style style strings, e.g. `"bold fg:#A6E22E bg:236"`.
//!
//! Grammar: whitespace-separated tokens. Attribute tokens are `bold`,
//! `dimmed`, `italic`, `underline`, `inverted`. Color tokens are `fg:<color>`
//! / `bg:<color>` where `<color>` is `#rrggbb`, a 0-255 ANSI index, a named
//! ANSI color, or a palette entry name (resolved before parsing).

use anstyle::{AnsiColor, Color, RgbColor, Style};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum StyleError {
    UnknownToken(String),
    UnknownColor(String),
}

impl std::fmt::Display for StyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StyleError::UnknownToken(t) => write!(f, "unknown style token: {t}"),
            StyleError::UnknownColor(c) => write!(f, "unknown color: {c}"),
        }
    }
}

impl std::error::Error for StyleError {}

pub fn parse_style(spec: &str, palette: &HashMap<String, String>) -> Result<Style, StyleError> {
    let mut style = Style::new();
    for token in spec.split_whitespace() {
        style = match token {
            "bold" => style.bold(),
            "dimmed" => style.dimmed(),
            "italic" => style.italic(),
            "underline" => style.underline(),
            "inverted" => style.invert(),
            _ => {
                if let Some(color) = token.strip_prefix("fg:") {
                    style.fg_color(Some(parse_color(color, palette)?))
                } else if let Some(color) = token.strip_prefix("bg:") {
                    style.bg_color(Some(parse_color(color, palette)?))
                } else {
                    return Err(StyleError::UnknownToken(token.to_string()));
                }
            }
        };
    }
    Ok(style)
}

fn parse_color(name: &str, palette: &HashMap<String, String>) -> Result<Color, StyleError> {
    // Palette entries are themselves color specs (single level, no recursion).
    let name = palette.get(name).map(String::as_str).unwrap_or(name);

    if let Some(hex) = name.strip_prefix('#') {
        return parse_hex(hex).ok_or_else(|| StyleError::UnknownColor(name.to_string()));
    }
    if let Ok(index) = name.parse::<u8>() {
        return Ok(Color::Ansi256(index.into()));
    }
    let ansi = match name {
        "black" => AnsiColor::Black,
        "red" => AnsiColor::Red,
        "green" => AnsiColor::Green,
        "yellow" => AnsiColor::Yellow,
        "blue" => AnsiColor::Blue,
        "purple" | "magenta" => AnsiColor::Magenta,
        "cyan" => AnsiColor::Cyan,
        "white" => AnsiColor::White,
        "bright-black" => AnsiColor::BrightBlack,
        "bright-red" => AnsiColor::BrightRed,
        "bright-green" => AnsiColor::BrightGreen,
        "bright-yellow" => AnsiColor::BrightYellow,
        "bright-blue" => AnsiColor::BrightBlue,
        "bright-purple" | "bright-magenta" => AnsiColor::BrightMagenta,
        "bright-cyan" => AnsiColor::BrightCyan,
        "bright-white" => AnsiColor::BrightWhite,
        _ => return Err(StyleError::UnknownColor(name.to_string())),
    };
    Ok(Color::Ansi(ansi))
}

fn parse_hex(hex: &str) -> Option<Color> {
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(RgbColor(r, g, b)))
}

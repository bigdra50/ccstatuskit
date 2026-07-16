//! A rendered piece of statusline output: text plus an optional style.

use anstyle::Style;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Segment {
    pub text: String,
    pub style: Style,
}

impl Segment {
    pub fn plain(text: impl Into<String>) -> Self {
        Segment {
            text: text.into(),
            style: Style::new(),
        }
    }

    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Segment {
            text: text.into(),
            style,
        }
    }

    /// Appends this segment to `out`, wrapping it in ANSI codes unless color
    /// is disabled or the style is empty.
    pub fn render_into(&self, out: &mut String, color: bool) {
        use std::fmt::Write;
        if color && self.style != Style::new() {
            let _ = write!(out, "{}{}{:#}", self.style, self.text, self.style);
        } else {
            out.push_str(&self.text);
        }
    }
}

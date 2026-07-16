//! Default palette (Monokai), ported from the original bash statusline.

use anstyle::{Color, RgbColor, Style};

const fn fg(r: u8, g: u8, b: u8) -> Style {
    Style::new().fg_color(Some(Color::Rgb(RgbColor(r, g, b))))
}

pub const GREEN: Style = fg(0xA6, 0xE2, 0x2E);
pub const YELLOW: Style = fg(0xE6, 0xDB, 0x74);
pub const BLUE: Style = fg(0x66, 0xD9, 0xEF);
pub const MAGENTA: Style = fg(0xAE, 0x81, 0xFF);
pub const CYAN: Style = fg(0xA1, 0xEF, 0xE4);
pub const ORANGE: Style = fg(0xFD, 0x97, 0x1F);
pub const RED: Style = fg(0xF9, 0x26, 0x72);
pub const WHITE: Style = fg(0xF8, 0xF8, 0xF2);
pub const GRAY: Style = fg(0x75, 0x71, 0x5E);

use core::fmt;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Color {
    None = 0,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Purple = 35,
    Cyan = 36,
    White = 37,
    Gray = 90,
}

#[derive(Clone, Copy)]
pub struct ColoredString<'a> {
    pub fg: Color,
    pub bg: Color,
    pub data: &'a str,
}

impl<'a> ColoredString<'a> {
    pub fn fg(&mut self, fg: Color) -> ColoredString {
        self.fg = fg;
        *self
    }
    pub fn bg(&mut self, bg: Color) -> ColoredString {
        self.bg = bg;
        *self
    }
}

impl<'a> fmt::Display for ColoredString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B[0;{}m{}\x1B[0m", self.fg as u8, self.data)
    }
}

pub trait Colored {
    fn fg(&self, fg: Color) -> ColoredString;
    fn bg(&self, bg: Color) -> ColoredString;
}

impl Colored for &str {
    fn fg(&self, fg: Color) -> ColoredString {
        ColoredString {
            data: self,
            fg,
            bg: Color::None,
        }
    }
    fn bg(&self, bg: Color) -> ColoredString {
        ColoredString {
            data: self,
            fg: Color::None,
            bg,
        }
    }
}

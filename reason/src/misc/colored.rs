use crate::serial::println;
use core::fmt::{self, UpperHex};

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
pub struct Colored<T>
where
    T: fmt::Display,
{
    pub fg: Color,
    pub bg: Color,
    pub data: T,
}

impl<T> fmt::Display for Colored<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B[0;{}m{}\x1B[0m", self.fg as u8, self.data)
    }
}

pub trait Colorize<T>
where
    T: fmt::Display + Copy,
{
    fn fg(&self, fg: Color) -> Colored<T>;
    fn bg(&self, bg: Color) -> Colored<T>;

    fn cyan(&self) -> Colored<T>;
    fn blue(&self) -> Colored<T>;
    fn yellow(&self) -> Colored<T>;
    fn red(&self) -> Colored<T>;
    fn green(&self) -> Colored<T>;
    fn purple(&self) -> Colored<T>;
    fn white(&self) -> Colored<T>;
    fn gray(&self) -> Colored<T>;
}

macro_rules! color_fn {
    ($name:ident, $color:ident) => {
        fn $name(&self) -> Colored<T> {
            Colored {
                data: *self,
                fg: Color::$color,
                bg: Color::None,
            }
        }
    };
}

impl<T> Colorize<T> for T
where
    T: fmt::Display + Copy,
{
    fn fg(&self, fg: Color) -> Colored<T> {
        Colored {
            data: *self,
            fg,
            bg: Color::None,
        }
    }

    fn bg(&self, bg: Color) -> Colored<T> {
        Colored {
            data: *self,
            fg: Color::None,
            bg,
        }
    }

    color_fn!(cyan, Cyan);
    color_fn!(blue, Blue);
    color_fn!(gray, Gray);
    color_fn!(white, White);
    color_fn!(yellow, Yellow);
    color_fn!(red, Red);
    color_fn!(green, Green);
    color_fn!(purple, Purple);
}

impl<T> fmt::UpperHex for Colored<T>
where
    T: fmt::UpperHex + fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B[0;{}m", self.fg as u8)?;
        <T as fmt::UpperHex>::fmt(&self.data, f)?;
        write!(f, "\x1B[0m")
    }
}

impl<T> fmt::LowerHex for Colored<T>
where
    T: fmt::LowerHex + fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B[0;{}m", self.fg as u8)?;
        <T as fmt::LowerHex>::fmt(&self.data, f)?;
        write!(f, "\x1B[0m")
    }
}

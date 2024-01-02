#![allow(unused_macros, unused_imports)]

macro_rules! debug {
    ($($arg:tt)*) => ($crate::serial::print!("[\x1b[33m DEBUG \x1B[0m]: {}\n", format_args!($($arg)*)));
}

macro_rules! warning {
    ($($arg:tt)*) => ($crate::serial::print!("[\x1b[31m WARN \x1B[0m]: {}\n", format_args!($($arg)*)));
}

macro_rules! info {
    ($($arg:tt)*) => ($crate::serial::print!("[\x1b[36m INFO \x1B[0m]: {}\n", format_args!($($arg)*)));
}

pub(crate) use debug;
pub(crate) use warning;
pub(crate) use info;

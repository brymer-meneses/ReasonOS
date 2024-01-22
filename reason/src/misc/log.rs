macro_rules! debug {
    ($($arg:tt)*) => {
        {
            use $crate::misc::colored::Colored;
            use $crate::misc::colored::Color;
            ($crate::serial::print!("{} {} {} {}\n", "[".fg(Color::Gray), "DEBG".fg(Color::Yellow), "]:".fg(Color::Gray),  format_args!($($arg)*)))
        }
    };
}

macro_rules! warning {
    ($($arg:tt)*) => {
        {
            use $crate::misc::colored::Colored;
            use $crate::misc::colored::Color;
            ($crate::serial::print!("{} {} {} {}\n", "[".fg(Color::Gray), "WARN".fg(Color::Red), "]:".fg(Color::Gray),  format_args!($($arg)*)))
        }
    };
}

macro_rules! info {
    ($($arg:tt)*) => {
        {
            use $crate::misc::colored::Colored;
            use $crate::misc::colored::Color;
            ($crate::serial::print!("{} {} {} {}\n", "[".fg(Color::Gray), "INFO".fg(Color::Blue), "]:".fg(Color::Gray),  format_args!($($arg)*)))
        }
    };
}

pub(crate) use debug;
pub(crate) use info;
pub(crate) use warning;

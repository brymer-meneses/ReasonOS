macro_rules! debug {
    ($($arg:tt)*) => {
        {
            use $crate::misc::colored::Colorize;
            ($crate::serial::print!("{} {} {} {}\n", "[".gray(), "DEBG".yellow(), "]:".gray(),  format_args!($($arg)*)))
        }
    };
}

macro_rules! warning {
    ($($arg:tt)*) => {
        {
            use $crate::misc::colored::Colorize;
            ($crate::serial::print!("{} {} {} {}\n", "[".gray(), "WARN".red(), "]:".gray(),  format_args!($($arg)*)))
        }
    };
}

macro_rules! info {
    ($($arg:tt)*) => {
        {
            use $crate::misc::colored::Colorize;
            ($crate::serial::print!("{} {} {} {}\n", "[".gray(), "INFO".blue(), "]:".gray(),  format_args!($($arg)*)))
        }
    };
}

pub(crate) use debug;
pub(crate) use info;
pub(crate) use warning;

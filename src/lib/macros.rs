#[macro_export]
macro_rules! error_println {
    ($($arg:tt)*) => {
        use colored::Colorize;
        eprintln!("{}", format!(">>>ERROR - {}", format!($($arg)*)).red());
    }
}

#[macro_export]
macro_rules! warning_println {
    ($($arg:tt)*) => {
        use colored::Colorize;
        println!("{}", format!(">>>WARNING - {}", format!($($arg)*)).yellow());
    }
}

pub use error_println;
pub use warning_println;

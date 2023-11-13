use std::{env, io, io::Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

pub struct SimpleLogger {
    log_level: LogLevel,
}

impl SimpleLogger {
    fn new(log_level: LogLevel) -> Self {
        SimpleLogger { log_level }
    }

    fn log(&self, level: LogLevel, message: &str) {
        if level <= self.log_level {
            let output = io::stderr();
            let mut handle = output.lock();

            writeln!(handle, "[{:?}] {}", level, message).expect("Failed to write to stderr");
        }
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warning, message);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }
}

// Initialize the logger based on the environment variable 'RUST_LOG_LEVEL'
pub fn init() -> SimpleLogger {
    let log_level = match env::var("RUST_LOG_LEVEL").unwrap_or_else(|_| "Info".to_string()).as_str() {
        "Debug" => LogLevel::Debug,
        "Info" => LogLevel::Info,
        "Warning" => LogLevel::Warning,
        "Error" => LogLevel::Error,
        _ => LogLevel::Info,
    };

    SimpleLogger::new(log_level)
}

// Singleton instance of the logger
static LOGGER: SimpleLogger = SimpleLogger { log_level: LogLevel::Info };

// Accessor function to get the logger instance
pub(crate) fn get_logger() -> &'static SimpleLogger {
    &LOGGER
}

// Define macros for logging at different levels
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::get_logger().info(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::get_logger().debug(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::get_logger().warn(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::get_logger().error(&format!($($arg)*));
    };
}

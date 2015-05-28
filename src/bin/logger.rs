/// Simple logging facilities using the log crate.

extern crate log;
use log::{LogRecord, LogLevel, LogMetadata, LogLevelFilter, SetLoggerError};

/// Outputs all log messages to stdout.
struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}

/// Create and install a new ConsoleLogger as the default logger.
pub fn init_console_logger() -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Info);
        Box::new(ConsoleLogger)
    })
}

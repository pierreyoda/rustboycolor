/// Simple logging facilities using the log crate.
extern crate log;
use log::{Level, Log, Metadata, Record, SetLoggerError};

const LOG_LEVEL: Level = Level::Trace;

/// Outputs all log messages to stdout.
struct ConsoleLogger;

impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= LOG_LEVEL
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

/// Create and install a new ConsoleLogger as the default logger.
pub fn init_console_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&ConsoleLogger)?;
    log::set_max_level(LOG_LEVEL.to_level_filter());
    Ok(())
}

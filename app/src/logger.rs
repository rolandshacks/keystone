//!
//! Terminal logger
//!
//! # Example
//!
//! ```
//! use log::{*};
//! static LOGGER: api::DefaultLogger = api::default_logger();
//! fn main() {
//!     api::init_logger(logger, api::LogLevel::Trace);
//!     trace!("trace");
//!     debug!("debug");
//!     info!("info");
//!     warn!("warn");
//!     error!("error");
//! }
//! ```
//!

use log::{Log, Metadata, Record};

pub struct DefaultLogger;

static SHOW_TIMESTAMP: bool = false;
static mut START_TIME: Option<std::time::Instant> = None;

impl log::Log for DefaultLogger {

    fn enabled(&self, metadata: &Metadata) -> bool {
        let max_level = log::max_level();
        return metadata.level() <= max_level;
    }

    fn log(&self, record: &Record) {

        if !self.enabled(record.metadata()) {
            return;
        }

        let mut timestamp: u128 = 0;

        if SHOW_TIMESTAMP {
            unsafe {
                timestamp = START_TIME.unwrap().elapsed().as_millis();
            }
        }

        let level = record.level();

        let module = match record.module_path() {
            Some(s) => s,
            None => ""
        };

        let location = match (record.file(), record.line()) {
            (Some(s), Some(l)) => format!("{}({}): ", s, l),
            (Some(s), None) => format!("{}: ", s),
            _ => String::from("")
        };

        let style_on = match level {
            log::Level::Trace => "\x1b[35m",
            log::Level::Debug => "\x1b[34m",
            log::Level::Info => "\x1b[32m",
            log::Level::Warn => "\x1b[33m",
            log::Level::Error => "\x1b[31m"
        };

        if SHOW_TIMESTAMP {
            println!("{:<6} {}{}{:5}\x1b[0m \x1b[97m{}\x1b[0m > {}", timestamp, location, style_on, level, module, record.args());
        } else {
            println!("{}{}{:5}\x1b[0m \x1b[97m{}\x1b[0m > {}", location, style_on, level, module, record.args());
        }

    }

    fn flush(&self) {}
}

pub const fn default() -> DefaultLogger {
    DefaultLogger{}
}

pub fn init(logger: &'static dyn Log, log_level: log::LevelFilter) {
    unsafe {
        START_TIME = Some(std::time::Instant::now());
    }

    let _ = log::set_logger(logger)
                .map(|()| log::set_max_level(log_level));
}

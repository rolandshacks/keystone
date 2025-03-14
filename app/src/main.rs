//!
//! Keystone
//!

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use application::Application;

use config::{ASIO_DEVICE_NAME, VST_CLSID};
use error::Error;
use log::{*};
use logger::DefaultLogger;

mod constants;
mod logger;
mod utils;
mod error;
mod config;
mod application;
mod instance;
mod plugin;
mod registry;
mod edit_controller;
mod host;
mod audio_processor;
mod parameters;
mod events;
mod instrument;
mod session;
mod stream;
mod view;
mod context;
mod audio;
mod time;
mod painter;
mod window;

pub const fn default_logger() -> DefaultLogger {
    crate::logger::default()
}
pub type LogLevel = log::LevelFilter;

pub fn init_logger(logger: &'static dyn log::Log, log_level: LogLevel) {
    crate::logger::init(logger, log_level)
}

fn run() -> Result<(), Error> {
    trace!("run");

    let mut app = Application::new()?;

    app.create_audio(ASIO_DEVICE_NAME)?;
    app.create_window()?;
    app.load_instrument(VST_CLSID)?;
    app.run()?;
    app.unload_instrument()?;
    app.close_window()?;
    app.close_audio()?;
    app.dispose();

    Ok(())
}

fn main() {

    init_logger(&DefaultLogger, LogLevel::Trace);

    debug!("Keystone - STARTED");

    match run() {
        Ok(_) => {},
        Err(e) => {
            error!("Error: {}", e.message());
        }
    }

    debug!("Keystone - DONE");
}

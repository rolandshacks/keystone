//!
//! Time
//!

use std::sync::{Arc, Mutex};

use crate::error::Error;

pub struct TimingContext {
    pub start_time: std::time::SystemTime,
    pub system_time: u64,
    pub tempo: f64
}

pub type SharedTimingContext = Arc<Mutex<TimingContext>>;

impl Default for TimingContext {
    fn default() -> Self {
        let start_time = std::time::SystemTime::now();
        let system_time = 0u64;

        Self {
            start_time,
            system_time,
            tempo: 120.0
        }
    }
}

impl TimingContext {
    pub fn update(&mut self) -> Result<(), Error> {
        self.system_time = match self.start_time.elapsed() {
            Ok(tm) => tm.as_nanos() as u64,
            Err(_) => 0
        };

        Ok(())
    }
}

pub struct Timing {
}

impl Timing {
    pub fn new() -> Result<SharedTimingContext, Error> {
        Ok(Arc::new(Mutex::new(TimingContext::default())))
    }
}

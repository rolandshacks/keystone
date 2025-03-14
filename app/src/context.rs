//!
//! Context
//!

use std::sync::{Arc, Mutex};

#[derive(Default, Debug)]
pub struct ApplicationContext {
    w: f64
}

pub type SharedApplicationContext = Arc<Mutex<ApplicationContext>>;

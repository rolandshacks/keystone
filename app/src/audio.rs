//!
//! Audio
//!

use log::{*};
use std::{sync::{Arc, Mutex}, time::Duration};
use crate::error::Error;
use std::ffi::c_void;

// Number of channels.
pub type ChannelCount = u16;

struct AudioContext {
    running: bool,
    stream: asio_sys::AsioStream,
}

unsafe impl Sync for AudioContext {}
unsafe impl Send for AudioContext {}

impl AudioContext {
    pub fn new(stream: asio_sys::AsioStream) -> Self
    {
        Self {
            running: false,
            stream,
        }
    }

    pub fn set_running(&mut self, status: bool) {
        self.running = status;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

pub struct AudioCallbackInfo {
    pub buffer0: *mut c_void,
    pub buffer1: *mut c_void,
    pub buffer_size: usize
}

#[derive(Clone, Debug)]
pub struct AudioFormatInfo {
    pub sample_rate: f64,
    pub num_channels: usize,
    pub buffer_size: usize
}

pub struct Audio {
    context: Arc<Mutex<AudioContext>>,
    asio: asio_sys::Asio,
    driver: Option<asio_sys::Driver>,
    asio_callback_id: Option<asio_sys::CallbackId>,
    format: AudioFormatInfo
}

impl Audio {
    pub fn new(name: &str, configured_sample_rate: f64, configured_buffer_size: usize) -> Result<Self, Error> {
        trace!("new");

        let asio = asio_sys::Asio::new();

        let driver = match asio.load_driver(name) {
            Ok(driver) => driver,
            Err(_) => {
                //eprintln!("failed to load driver: {:?}", e);
                return Err(Error::from("failed to load driver"));
            }
        };

        trace!("loaded driver: '{}'", name);

        let buffer_size = if configured_buffer_size > 0 {
            match driver.buffersize_range() {
                Ok((min_size, max_size)) => {
                    configured_buffer_size.clamp(min_size as usize, max_size as usize)
                },
                Err(_) => {
                    0usize
                }
            }
        } else {
            0usize
        };

        let num_channels = 2;

        let buffer_size_override: Option<i32> = if buffer_size > 0 { Some(buffer_size as i32) } else { None };

        if driver.can_sample_rate(configured_sample_rate).is_ok() {
            match driver.set_sample_rate(configured_sample_rate) {
                Ok(_) => {},
                Err(_) => {
                    //eprintln!("failed to set sample rate: {:?}", e);
                    return Err(Error::from("failed to set sample rate"));
                }
            }
        }

        let stream = match driver.prepare_output_stream(None, num_channels, buffer_size_override) {
            Ok(streams) => {
                match streams.output {
                    Some(output_stream) => output_stream,
                    None => {
                        return Err(Error::from("failed to prepare output stream"));
                    }
                }
            },
            Err(_) => {
                return Err(Error::from("failed to prepare output stream"));
            }
        };

        let sample_type = driver.output_data_type().unwrap();
        let sample_rate = driver.sample_rate().unwrap();
        let buffer_size = stream.buffer_size as usize;
        trace!("asio sample data format: {:?}", sample_type);
        trace!("asio sample buffer size: {}", buffer_size);
        trace!("asio sample rate: {}", sample_rate);

        let context = Arc::new(Mutex::new(AudioContext::new(stream)));

        let format_info = AudioFormatInfo {
            sample_rate,
            num_channels,
            buffer_size
        };

        Ok(Self {
            context,
            asio,
            driver: Some(driver),
            asio_callback_id: None,
            format: format_info
        })

    }

    pub fn start<F>(&mut self, mut callback: F) -> Result<(), Error>
    where
        F: 'static + FnMut(&AudioCallbackInfo) + Send
    {
        if self.driver.is_none() {
            return Err(Error::from("driver not initialized"));
        }

        let driver = self.driver.as_mut().unwrap();
        let context = &mut self.context;

        context.lock().unwrap().set_running(true);



        let callback_id = {
            let callback_context = context.clone();

            driver.add_callback(move |callback_info| {
                let buffer_index = callback_info.buffer_index as usize;
                let _tm = Self::get_callback_time(callback_info);

                let audio_callback_info = match callback_context.lock() {
                    Ok(context) => {
                        if !context.is_running() { return };

                        let buffer0 = context.stream.buffer_infos[0].buffers[buffer_index];
                        let buffer1 = context.stream.buffer_infos[1].buffers[buffer_index];
                        let buffer_size = context.stream.buffer_size as usize;

                        AudioCallbackInfo {
                            buffer0,
                            buffer1,
                            buffer_size
                        }
                    },
                    Err(_) => { return; }
                };

                callback(&audio_callback_info);

            })
        };

        self.asio_callback_id = Some(callback_id);

        match driver.start() {
            Ok(_) => {},
            Err(_) => {
                return Err(Error::from("failed to start driver"));
            }
        }

        Ok(())

    }

    pub fn stop(&mut self) -> Result<(), Error> {
        trace!("stop");

        match self.driver.as_mut() {
            Some(driver) => {
                match self.asio_callback_id.take() {
                    Some(callback_id) => {
                        driver.remove_callback(callback_id);
                    },
                    None => {}
                };

                let _ = driver.stop();

            },
            None => {
                self.asio_callback_id = None;
            }
        };

        Ok(())
    }

    pub fn dispose(&mut self) {
        trace!("dispose");

        let _ = self.stop();

        match self.driver.take() {
            Some(driver) => {
                let _ = driver.dispose_buffers();
                let _ = driver.destroy();
            },
            None => {}
        };
    }

    pub fn get_format(&self) -> &AudioFormatInfo {
        &self.format
    }

    pub fn list_drivers() {
        trace!("list drivers");

        let asio = asio_sys::Asio::new();
        for name in asio.driver_names() {
            println!("Driver: {:?}", name);
        }
    }

    pub fn get_callback_time(callback_info: &asio_sys::CallbackInfo) -> Duration {
        let nanos = ((callback_info.system_time.hi as u64) << 32) | (callback_info.system_time.lo as u64)  ;
        Duration::from_nanos(nanos)
    }
}

//!
//! Application
//!


use core::slice;
use std::ptr::null_mut;

use log::{*};
use vst3_sys::{base::kResultOk, vst::{AudioBusBuffers, IAudioProcessor}};
use crate::{audio::{Audio, AudioFormatInfo}, config::{ASIO_BUFFER_SIZE, ASIO_SAMPLE_RATE}, error::Error, host::Host, instance::Instance, instrument::Instrument, registry::Registry, time::{SharedTimingContext, Timing}, window::Window};

pub struct Application {
    registry: Registry,
    host: Host,
    timing: SharedTimingContext,
    audio: Option<Audio>,
    instance: Option<Instance>,
    instrument: Option<Instrument>,
    window: Option<Box<Window>>
}

impl Application {
    pub fn new() -> Result<Self, Error> {
        trace!("new");

        let mut registry = Registry::new();
        registry.init()?;

        let host = Host::new()?;

        let timing = Timing::new()?;

        let app = Self {
            registry,
            host,
            timing,
            audio: None,
            instance: None,
            instrument: None,
            window: None
        };

        Ok(app)
    }

    pub fn dispose(&mut self) {

        let _ = self.unload_instrument();
        let _ = self.close_window();
        let _ = self.close_audio();

        self.host.dispose();
        self.registry.dispose();
    }

    pub fn load_instrument(&mut self, classid: &str) -> Result<(), Error> {
        trace!("load instrument");

        if self.audio.is_none() {
            return Err(Error::from("audio device not initialized"));
        }

        trace!("create instance");

        let instance = self.registry.create_class_instance(classid)?;

        crate::utils::trace_ref(&instance.instance);

        trace!("initialize instance");

        instance.initialize(&self.host)?;

        crate::utils::trace_ref(&instance.instance);

        trace!("create instrument");

        let instrument = Instrument::new(&instance, &self.host, self.audio.as_mut().unwrap())?;

        trace!("create window");

        match self.window.as_mut() {
            Some(window) => {
                trace!("create view");
                let view = instrument.create_view()?;

                trace!("attach view");
                window.attach_view(view)?;
            },
            None => {}
        };

        self.instance = Some(instance);
        self.instrument = Some(instrument);

        self.set_active(true)?;

        Ok(())
    }

    pub fn unload_instrument(&mut self) -> Result<(), Error> {
        trace!("unload instrument");

        let _ = self.set_active(false);

        match &self.instance {
            Some(instance) => {
                crate::utils::trace_ref(&instance.instance);
            },
            None => {}
        };

        match self.window.as_mut() {
            Some(window) => {
                trace!("detach view");
                let _ = window.detach_view();
            },
            None => {}
        };

        match &self.instance {
            Some(instance) => {
                let _ = instance.set_active(false);
                crate::utils::trace_ref(&instance.instance);
            },
            None => {}
        };

        match self.instrument.take() {
            Some(instrument) => {
                trace!("dispose instrument");
                /*
                match Arc::try_unwrap(instrument) {
                    Ok(instrument_lock) => {
                        let instrument = instrument_lock.into_inner().unwrap();
                        instrument.dispose();
                    },
                    Err(_) => {
                        // still other references
                    }
                };
                */
                instrument.dispose();
            },
            None => {}
        };

        match &self.instance {
            Some(instance) => {
                crate::utils::trace_ref(&instance.instance);
            },
            None => {}
        };

        match self.instance.take() {
            Some(mut instance) => {
                crate::utils::trace_ref(&instance.instance);

                trace!("deactivate instance");
                let _ = instance.set_active(false);

                trace!("terminate instance");
                let _ = instance.terminate();

                trace!("dispose instance");
                instance.dispose();

                trace!("unref instance");
                let _ = self.registry.unref_class_instance(instance);
            },
            None => {}
        };

        Ok(())
    }

    pub fn create_window(&mut self) -> Result<(), Error> {
        trace!("create window");
        let mut window = Window::new("Keystone", 800, 600, true)?;
        //window.start_timer(&Duration::from_millis(1000))?;
        window.show();
        self.window = Some(window);
        Ok(())
    }

    pub fn close_window(&mut self) -> Result<(), Error> {
        trace!("close window");
        match self.window.take() {
            Some(mut window) => {
                window.stop_timer()?;
                window.dispose();
            },
            None => {}
        };
        Ok(())
    }

    pub fn set_active(&mut self, active: bool) -> Result<(), Error> {
        trace!("set active");

        self.set_processing(false)?;

        match &self.instance {
            Some(instance) => {
                let _ = instance.set_active(active);
            },
            None => {}
        };

        if active {
            self.set_processing(active)?;
        }

        Ok(())

    }

    fn set_processing(&mut self, processing: bool) -> Result<(), Error> {
        match self.instrument.as_mut() {
            Some(instrument) => {
                /*
                match instrument.lock() {
                    Ok(mut instrument) => {
                        instrument.set_processing(processing)?;
                    },
                    Err(_) => {}
                }
                */
                instrument.set_processing(processing)?;
            },
            None => {}
        }

        Ok(())
    }

    pub fn create_audio(&mut self, name: &str) -> Result<(), Error> {
        trace!("start audio");

        let _ = self.close_audio();

        let sample_rate = ASIO_SAMPLE_RATE;
        let buffer_size = ASIO_BUFFER_SIZE;

        let audio = Audio::new(name, sample_rate, buffer_size)?;

        self.audio = Some(audio);

        Ok(())
    }

    pub fn close_audio(&mut self) -> Result<(), Error> {
        trace!("stop audio");

        match self.audio.take() {
            Some(mut audio) => {
                audio.stop()?;
                audio.dispose();
            },
            None => {}
        };

        Ok(())
    }

    pub fn get_audio_format(&self) -> Option<&AudioFormatInfo> {
        match self.audio.as_ref() {
            Some(audio) => {
                Some(audio.get_format())
            },
            None => None
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        trace!("run");

        let mut context = match self.instrument.as_mut() {
            Some(instrument) => {
                Some(instrument.get_context().clone())
            },
            None => None
        };

        if self.audio.is_some() {
            self.audio.as_mut().unwrap().start(move |callback_info| {

                //trace!("audio callback");

                match context.as_mut() {
                    Some(context) => {
                        match context.lock() {
                            Ok(mut context) => {
                                let audio_processor_intf = &context.audio_processor.audio_processor.clone();
                                let process_data = &mut context.process_data;

                                Self::process_data(audio_processor_intf, process_data, callback_info);

                                /*
                                process_data.num_samples = callback_info.buffer_size as i32;

                                let audio_buffers = [
                                    callback_info.buffer0,
                                    callback_info.buffer1
                                ].as_mut_ptr();

                                let mut output_buffers = AudioBusBuffers {
                                    num_channels: 2,
                                    silence_flags: 0x0,
                                    buffers: audio_buffers
                                };

                                process_data.outputs = &mut output_buffers;

                                let result = unsafe {
                                    audio_processor_intf.process(process_data as *mut _)
                                };

                                if result != kResultOk {
                                    trace!("audio processor processing failed");
                                }
                                */
                            },
                            Err(_) => {}
                        };
                    },
                    None => {}
                }

            })?
        }

        if self.window.is_some() {
            self.window.as_ref().unwrap().event_loop();
        }

        if self.audio.is_some() {
            let _ = self.audio.as_mut().unwrap().stop();
        }

        Ok(())
    }

    fn process_data(audio_processor_intf: &vst3_com::VstPtr<dyn IAudioProcessor>, process_data: &mut vst3_sys::vst::ProcessData, callback_info: &crate::audio::AudioCallbackInfo) {
        process_data.num_samples = callback_info.buffer_size as i32;

        let mut audio_buffers = [
            callback_info.buffer0,
            callback_info.buffer1
        ];

        let audio_buffers_ptr = audio_buffers.as_mut_ptr();

        let mut output_buffers = AudioBusBuffers {
            num_channels: 2,
            silence_flags: 0x0,
            buffers: audio_buffers_ptr
        };

        process_data.outputs = &mut output_buffers;

        let mut input_buffers = AudioBusBuffers {
            num_channels: 0,
            silence_flags: 0x0,
            buffers: null_mut()
        };

        process_data.inputs = &mut input_buffers;

        let result = unsafe { audio_processor_intf.process(process_data as *mut _) };
        if result != kResultOk {
            trace!("audio processor processing failed");
        }

        {
            let buffer_size = callback_info.buffer_size;

            for buffer_ptr in audio_buffers {
                let in_buffer = unsafe { slice::from_raw_parts_mut(buffer_ptr as *mut f32, buffer_size) };
                let out_buffer = unsafe { slice::from_raw_parts_mut(buffer_ptr as *mut u32, buffer_size) };

                for i in 0..buffer_size {
                    let a = in_buffer[i];

                    let s = ((if a >= 0.0 {
                        a
                    } else {
                        2.0+a
                    }) * 32767.0) as u32;

                    let s32 = (s & 0xffff) << 16;

                    out_buffer[i] = s32;
                }
            }

            /*

            let buffer_size = callback_info.buffer_size;
            let output_buffer0 = unsafe { slice::from_raw_parts_mut(callback_info.buffer0 as *mut SampleType, buffer_size) };
            let output_buffer1 = unsafe { slice::from_raw_parts_mut(callback_info.buffer1 as *mut SampleType, buffer_size) };

            let mut w: f64 = 0.0;
            let volume = 0.5;

            for i in 0..buffer_size {
                w += 0.012;
                if w >= 1.0 { w -= 1.0; }

                let a = f64::sin(w * 3.1415 * 2.0) * volume;
                //let a = (w * 2.0 - 1.0) * volume;

                let s = ((if a >= 0.0 {
                    a
                } else {
                    2.0+a
                }) * 32767.0) as SampleType;

                (((a + 1.0) * 32767.0) as SampleType);

                let s1 = (s & 0xffff) << 16;

                output_buffer0[i] = s1;
                output_buffer1[i] = s1;
            }
            */

        }

        /*
        type SampleType = u32;

        let buffer_size = callback_info.buffer_size;
        let output_buffer0 = unsafe { slice::from_raw_parts_mut(callback_info.buffer0 as *mut SampleType, buffer_size) };
        let output_buffer1 = unsafe { slice::from_raw_parts_mut(callback_info.buffer1 as *mut SampleType, buffer_size) };

        let mut w: f64 = 0.0;
        let volume = 0.5;

        for i in 0..buffer_size {
            w += 0.012;
            if w >= 1.0 { w -= 1.0; }

            let a = f64::sin(w * 3.1415 * 2.0) * volume;
            //let a = (w * 2.0 - 1.0) * volume;

            let s = ((if a >= 0.0 {
                a
            } else {
                2.0+a
            }) * 32767.0) as SampleType;

            (((a + 1.0) * 32767.0) as SampleType);

            let s1 = (s & 0xffff) << 16;

            output_buffer0[i] = s1;
            output_buffer1[i] = s1;
        }
        */

    }

}

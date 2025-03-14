
use log::{*};
use vst3_com::VstPtr;
use vst3_sys::{base::kResultOk, vst::{BusDirections, Chord, FrameRate, IAudioProcessor, ProcessContext, ProcessModes, ProcessSetup, SymbolicSampleSizes}};

use crate::{audio::{Audio, AudioFormatInfo}, error::Error, instance::Instance, instrument::ProcessContextFlags};

const DEFAULT_AUDIO_BUFFER_SIZE: usize = 128;
const DEFAULT_SAMPLE_RATE: f64 = 48000.0;

pub struct AudioContext {
    pub audio_format: AudioFormatInfo,
    pub samples_per_block: usize,
    pub latency_samples: usize,
    pub process_context: Box<ProcessContext>,
}

pub struct AudioProcessor {
    pub audio_processor: VstPtr<dyn IAudioProcessor>,
    pub context: AudioContext
}

impl Drop for AudioProcessor {
    fn drop(&mut self) {
        trace!("drop AudioProcessor");
    }
}

impl AudioProcessor {
    pub fn new(instance: &Instance, audio: &Audio) -> Result<Self, Error> {
        trace!("new");

        let audio_processor = instance.query_audio_processor_intf()?;
        let _ = unsafe { audio_processor.set_processing(0) };

        let samples_per_block = DEFAULT_AUDIO_BUFFER_SIZE;
        let latency_samples: usize = 0;

        let audio_format = audio.get_format();

        let process_context = Self::create_process_context(audio_format)?;

        let context = AudioContext {
            samples_per_block,
            latency_samples,
            process_context: Box::new(process_context),
            audio_format: audio_format.clone()
        };

        Ok(Self {
            audio_processor,
            context
        })
    }

    pub fn get_process_context(&mut self) -> *mut ProcessContext {
        self.context.process_context.as_mut()
    }

    pub fn get_format(&self) -> &AudioFormatInfo {
        &self.context.audio_format
    }

    pub fn get_audio_processor_intf(&self) -> &VstPtr<dyn IAudioProcessor> {
        &self.audio_processor
    }

    fn create_process_context(audio_format: &AudioFormatInfo) -> Result<ProcessContext, Error> {

        let chord = Chord {
            key_note: 0,
            root_note: 0,
            chord_mask: 0x0
        };

        let frame_rate = FrameRate {
            frames_per_second: 0,
            flags: 0x0,
        };

        let process_context = ProcessContext {
            state: ProcessContextFlags::kPlaying as u32,
            sample_rate: audio_format.sample_rate,
            project_time_samples: 0,
            system_time: 0,
            continuous_time_samples: 0,
            project_time_music: 0.0,
            bar_position_music: 0.0,
            cycle_start_music: 0.0,
            cycle_end_music: 0.0,
            tempo: 120.0,
            time_sig_num: 0,
            time_sig_den: 0,
            chord,
            smpte_offset_subframes: 0,
            frame_rate,
            samples_to_next_clock: audio_format.buffer_size as i64
        };

        Ok(process_context)
    }

    pub fn dispose(&mut self) {
        let _ = self.set_processing(false);
    }

    pub fn setup_processing(&mut self) -> Result<(), Error> {

        let audio_format = &self.context.audio_format;

        let mut processing_setup = ProcessSetup {
            process_mode: ProcessModes::kRealtime as i32,
            symbolic_sample_size: SymbolicSampleSizes::kSample32 as i32,
            max_samples_per_block: audio_format.buffer_size as i32,
            sample_rate: audio_format.sample_rate
        };

        let result = unsafe { self.audio_processor.setup_processing(&mut processing_setup) };
        if result != kResultOk {
            return Err(Error::from("failed to setup processing"))
        }

        Ok(())
    }

    pub fn set_processing(&mut self, enable: bool) -> Result<(), Error> {

        let context = &mut self.context;

        if enable {
            context.latency_samples = Self::get_latency_samples(&self.audio_processor);
        }

        let _ = unsafe { self.audio_processor.set_processing(if enable { 1 } else { 0 }) };

        Ok(())
    }

    /*
    pub fn process(&self, process_data: &mut ProcessData, time: i64) -> Result<(), Error> {

        context.process_context.state |= ProcessContextFlags::kSystemTimeValid as u32;
        context.process_context.system_time = time;

        process_data.context = &mut context.process_context as *mut ProcessContext;

        let result = unsafe { context.audio_processor.process(process_data) };
        if result != kResultOk {
            return Err(Error::from("failed to process data"))
        }

        Ok(())
    }
    */

    pub fn get_bus_input_arrangement(&self) -> u64 {
        let mut bus_arrangement: u64 = 0;
        unsafe {
            if self.audio_processor.get_bus_arrangement(BusDirections::kInput as i32, 0, &mut bus_arrangement) == kResultOk { bus_arrangement } else { 0 }
        }
    }

    pub fn set_bus_input_arrangements(&self, bus_arrangement: u64) -> bool {
        let mut value = bus_arrangement;
        unsafe {
            if self.audio_processor.set_bus_arrangements(&mut value, 1, &mut value, 0) == kResultOk { true } else { false }
        }
    }

    pub fn get_bus_output_arrangement(&self) -> u64 {
        let mut bus_arrangement: u64 = 0;
        unsafe {
            if self.audio_processor.get_bus_arrangement(BusDirections::kOutput as i32, 0, &mut bus_arrangement) == kResultOk { bus_arrangement } else { 0 }
        }
    }

    pub fn set_bus_output_arrangements(&self, bus_arrangement: u64) -> bool {
        let mut value = bus_arrangement;
        unsafe {
            if self.audio_processor.set_bus_arrangements(&mut value, 0, &mut value, 1) == kResultOk { true } else { false }
        }
    }

    fn can_process_sample_size_32(audio_processor: &VstPtr<dyn IAudioProcessor>) -> bool {
        unsafe { audio_processor.can_process_sample_size(SymbolicSampleSizes::kSample32 as i32) == kResultOk }
    }

    fn can_process_sample_size_64(audio_processor: &VstPtr<dyn IAudioProcessor>) -> bool {
        unsafe { audio_processor.can_process_sample_size(SymbolicSampleSizes::kSample64 as i32) == kResultOk }
    }

    fn get_latency_samples(audio_processor: &VstPtr<dyn IAudioProcessor>) -> usize {
        unsafe { audio_processor.get_latency_samples() as usize }
    }

    fn get_tail_samples(audio_processor: &VstPtr<dyn IAudioProcessor>) -> usize {
        unsafe { audio_processor.get_tail_samples() as usize }
    }

}

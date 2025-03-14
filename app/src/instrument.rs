use log::{*};
use std::{ptr::null_mut, sync::{Arc, Mutex}};
use vst3_sys::{base::IUnknown, vst::{AudioBusBuffers, Event, IEditController, IEventList, IProcessContextRequirements, ProcessData, ProcessModes, SymbolicSampleSizes}};
use crate::{audio::Audio, audio_processor::AudioProcessor, edit_controller::EditController, error::Error, events::EventList, host::Host, instance::Instance, parameters::ParameterChanges, stream::ByteStream, view::View};

//const DEFAULT_AUDIO_BUFFER_SIZE: usize = 128;
//const DEFAULT_SAMPLE_RATE: f64 = 48000.0;
const ENABLE_DUMP_BUFFER: bool = false;

pub enum ProcessContextFlags {
    kPlaying = 1<<1,
    kCycleActive = 1<<2,
    kRecording = 1<<3,
    kSystemTimeValid = 1<<8,
    kProjectTimeMusicValid = 1 << 9,
    kTempoValid = 1 << 10,
    kBarPositionValid = 1 << 11,
    kCycleValid = 1 << 12,
    kTimeSigValid = 1 << 13,
    kSmpteValid = 1 << 14,
    kClockValid = 1 << 15,
    kContTimeValid = 1 << 17,
    kChordValid = 1 << 18
}

pub struct InstrumentContext {
    pub process_data: Box<ProcessData>,
    pub audio_processor: AudioProcessor
}

unsafe impl Sync for InstrumentContext {}
unsafe impl Send for InstrumentContext {}

pub struct Instrument {
    controller: EditController,
    input_param_changes: Box<ParameterChanges>,
    input_event_list: Box<EventList>,
    state_stream: Box<ByteStream>,
    context: Arc<Mutex<InstrumentContext>>
}

impl Drop for Instrument {
    fn drop(&mut self) {
        trace!("drop instrument");
    }
}

impl Instrument {
    pub fn new(instance: &Instance, host: &Host, audio: &Audio) -> Result<Self, Error> {
        trace!("new");

        crate::utils::trace_ref::<dyn IUnknown>(&instance.instance);

        let process_context_requirements = match instance.query_process_context_requirements_intf() {
            Ok(i_process_context_requirements) => {
                unsafe {
                    let flags = i_process_context_requirements.get_process_context_requirements();
                    i_process_context_requirements.release();
                    flags
                }
            },
            Err(_) => {
                0u32
            }
        };

        trace!("process context requirements: {:032b}", process_context_requirements);
        crate::utils::trace_ref::<dyn IUnknown>(&instance.instance);

        trace!("create audio processor");
        let mut audio_processor = AudioProcessor::new(&instance, audio)?;
        crate::utils::trace_ref::<dyn IUnknown>(&instance.instance);

        trace!("create edit controller");
        let mut controller = EditController::new(&instance)?;
        crate::utils::trace_ref::<dyn IUnknown>(&instance.instance);
        controller.initialize(host)?;

        trace!("create stream");
        let state_stream = ByteStream::new();
        crate::utils::trace_ref::<dyn IUnknown>(&instance.instance);

        let mut input_param_changes = ParameterChanges::new();
        let mut input_event_list = EventList::new();
        unsafe { input_event_list.get_event_count() };

        let mut process_data = Self::create_process_data(&mut input_param_changes, &mut input_event_list, &mut audio_processor)?;
        process_data.context = audio_processor.context.process_context.as_mut();

        let context = InstrumentContext {
            process_data: Box::new(process_data),
            audio_processor
        };

        let instrument = Self {
            controller,
            input_param_changes,
            input_event_list,
            state_stream,
            context: Arc::new(Mutex::new(context))
        };

        Ok(instrument)
    }

    pub fn dispose(mut self) {
        trace!("dispose");

        crate::utils::trace_ref::<dyn IEditController>(&self.controller.controller);
        self.controller.dispose();
        crate::utils::trace_ref::<dyn IEditController>(&self.controller.controller);

        match self.context.lock() {
            Ok(mut context) => {
                context.audio_processor.dispose();
            }
            Err(_) => {}
        };

    }

    /*
    pub fn init(&mut self, samples_per_block: usize, sample_rate: f64) -> Result<(), Error> {
        trace!("init");
        let _ = self.controller.set_component_state(&mut self.state_stream);

        match self.context.lock() {
            Ok(mut context) => {
                context.audio_processor.setup_processing(samples_per_block, sample_rate)?;
                context.audio_processor.set_processing(false)?;
            }
            Err(_) => {}
        };

        Ok(())
    }
    */

    pub fn push_event(&mut self, event: Event) -> Result<(), Error> {
        self.input_event_list.push_event(event)
    }

    pub fn clear_events(&mut self) -> Result<(), Error> {
        self.input_event_list.clear()
    }

    pub fn create_view(&self) -> Result<View, Error> {
        trace!("create view");
        let view = self.controller.create_view()?;
        Ok(view)
    }

    pub fn get_context(&self) -> &Arc<Mutex<InstrumentContext>> {
        &self.context
    }

    fn create_process_data(input_param_changes: &mut ParameterChanges, input_event_list: &mut EventList, audio_processor: &mut AudioProcessor) -> Result<ProcessData, Error> {
        let audio_buffers = [
            null_mut(),
            null_mut()
        ].as_mut_ptr();

        let mut output_buffers = AudioBusBuffers {
            num_channels: 2,
            silence_flags: 0x0,
            buffers: audio_buffers
        };

        let input_param_changes = input_param_changes.get_static_ptr();
        let output_param_changes = ParameterChanges::get_null_ptr();
        let input_events = input_event_list.get_static_ptr();
        let output_events = EventList::get_null_ptr();

        let audio_format = audio_processor.get_format();

        let data = ProcessData {
            process_mode: ProcessModes::kRealtime as i32,
            symbolic_sample_size: SymbolicSampleSizes::kSample32 as i32,
            num_samples: audio_format.buffer_size as i32,
            num_inputs: 0,
            num_outputs: 1,
            inputs: null_mut(),
            outputs: &mut output_buffers,
            input_param_changes,
            output_param_changes,
            input_events,
            output_events,
            context: audio_processor.get_process_context()
        };

        Ok(data)

    }

    pub fn set_processing(&mut self, enable: bool) -> Result<(), Error> {

        match self.context.lock() {
            Ok(mut context) => {
                context.audio_processor.set_processing(enable)?;
            }
            Err(_) => {}
        };

        Ok(())
    }

    /*
    fn dump_buffer(&self, buffer: &[u32; DEFAULT_AUDIO_BUFFER_SIZE]) {
        print!("[");
        for b in buffer {
            print!("{:08X}", b);
        }
        println!("]");
    }
    */

}

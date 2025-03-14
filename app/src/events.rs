use std::{ptr::null_mut, sync::Mutex};
use vst3_sys::{base::{kResultFalse, kResultOk, tresult}, utils::StaticVstPtr, vst::{Event, EventData, EventTypes, IEventList, IEventListVTable, NoteOffEvent, NoteOnEvent}, VST3};
use log::{*};

use crate::error::Error;

const MAX_EVENT_COUNT: usize = 256;

#[VST3(implements(IEventList))]
pub struct EventList {
    events: Mutex<Vec<Event>>
}

impl EventList {
    pub fn new() -> Box<Self> {
        let events = Mutex::new(Vec::new());
        let instance = Self::allocate(events);
        instance
    }

    pub fn get_static_ptr(&mut self) -> StaticVstPtr<dyn IEventList> {
        let static_vst_ptr: StaticVstPtr<dyn IEventList> = unsafe {
            std::mem::transmute(self as *mut _)
        };

        static_vst_ptr
    }

    pub fn get_null_ptr() -> StaticVstPtr<dyn IEventList> {
        let null_ptr: *mut IEventListVTable = null_mut();
        let ptr: StaticVstPtr<dyn IEventList> = unsafe {
            std::mem::transmute(null_ptr as *mut _)
        };
        return ptr
    }

    pub fn push_event(&mut self, event: Event) -> Result<(), Error> {
        trace!("push event");

        match self.events.lock() {
            Ok(mut e) => {

                if e.len() >= MAX_EVENT_COUNT {
                    return Err(Error::from("event list overflow"));
                }

                e.push(event);
            },
            Err(_) => {
                return Err(Error::from("failed to lock event list"));
            }
        }

        Ok(())
    }

    pub fn clear(&mut self)  -> Result<(), Error> {

        trace!("clear all events");

        match self.events.lock() {
            Ok(mut e) => {
                e.clear()
            },
            Err(_) => {
                return Err(Error::from("failed to clear event list"));
            }
        }

        Ok(())

    }

    pub fn new_note_on_event(pitch: i16, velocity: f32) -> EventData {
        EventData {
            note_on: NoteOnEvent {
                channel: 0,
                pitch,
                tuning: 0.0,
                velocity,
                length: 0,
                note_id: -1
            }
        }
    }

    pub fn new_note_off_event(pitch: i16) -> EventData {
        EventData {
            note_off: NoteOffEvent {
                channel: 0,
                pitch,
                velocity: 0.0,
                note_id: -1,
                tuning: 0.0
            }
        }
    }

    pub fn new_event(event_data: EventData, event_type: EventTypes) -> Event {
        Event {
            bus_index: 0,
            sample_offset: 0,
            ppq_position: 0.0,
            flags: 0x0,
            type_: event_type as u16,
            event: event_data,
        }
    }

}

impl IEventList for EventList {
    unsafe fn get_event_count(&self) -> i32 {
        //trace!("get event count");

        match self.events.lock() {
            Ok(e) => e.len() as i32,
            Err(_) => 0
        }
    }

    unsafe fn get_event(&self, index: i32, event_buffer_ptr: *mut Event) -> tresult {
        trace!("get event");

        if event_buffer_ptr.is_null() {
            return kResultFalse;
        }

        match self.events.lock() {
            Ok(e) => {
                if index < 0 || index as usize >= e.len() {
                    return kResultFalse;
                }

                event_buffer_ptr.copy_from(e.as_ptr(), 1);
            },
            Err(_) => {
                return kResultFalse;
            }
        }

        kResultOk
    }

    unsafe fn add_event(&self, _event_buffer_ptr: *mut Event) -> tresult {
        trace!("add event");
        return kResultFalse;
    }

}

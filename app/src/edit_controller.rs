
use std::ptr::null_mut;

use vst3_sys::{gui::{IPlugView, IPlugViewVTable}, utils::SharedVstPtr, vst::{IComponentHandler, ParameterInfo, String128}, VST3};
use vst3_com::*;
use vst3_sys::{base::*, vst::IEditController};

use crate::{config::ENABLE_COMPONENT_HANDLER, error::Error, host::Host, instance::Instance, stream::ByteStream, view::View};

use log::{*};

#[VST3(implements(IComponentHandler))]
pub struct ComponentHandler {
}

impl ComponentHandler {
    pub fn new() -> Box<Self> {
        let instance = Self::allocate();
        instance
    }

    pub fn get_shared_ptr(&mut self) -> SharedVstPtr<dyn IComponentHandler> {
        let shared_vst_ptr: SharedVstPtr<dyn IComponentHandler> = unsafe {
            std::mem::transmute(self as * mut _)
        };
        shared_vst_ptr
    }

    pub fn get_null_ptr() -> SharedVstPtr<dyn IComponentHandler> {
        let null_ptr: *mut IBStreamVTable = null_mut();
        let ptr: SharedVstPtr<dyn IComponentHandler> = unsafe {
            std::mem::transmute(null_ptr as *mut _)
        };
        return ptr
    }
}

impl IComponentHandler for ComponentHandler {
    unsafe fn begin_edit(&self, id: vst3_sys::vst::ParamID) -> tresult {
        trace!("component handler: begin edit {}", id);
        kResultFalse
    }

    unsafe fn end_edit(&self, id: vst3_sys::vst::ParamID) -> tresult {
        trace!("component handler: end edit {}", id);
        kResultFalse
    }

    unsafe fn perform_edit(&self, id: vst3_sys::vst::ParamID, value_normalized: vst3_sys::vst::ParamValue) -> tresult {
        trace!("component handler: perform edit {}, value normalized: {}", id, value_normalized);
        kResultFalse
    }

    unsafe fn restart_component(&self, flags: i32) -> tresult {
        trace!("component handler: restart component {}", flags);
        kResultFalse
    }
}

pub struct EditController {
    pub controller: VstPtr<dyn IEditController>,
    pub component_handler: Box<ComponentHandler>,
    pub is_instance: bool
}

impl Drop for EditController {
    fn drop(&mut self) {
        trace!("drop EditController");
    }
}

impl EditController {
    pub fn new(instance: &Instance) -> Result<Self, Error> {
        trace!("new");

        crate::utils::trace_ref::<dyn IUnknown>(&instance.instance);

        let component_handler = ComponentHandler::allocate();

        let (controller, is_instance) = match instance.query_edit_controller_intf() {
            Ok(intf) => {
                (intf, false)
            },
            Err(_) => {
                match instance.create_edit_controller_intf() {
                    Ok(intf) => {
                        (intf, true)
                    },
                    Err(e) => {
                        return Err(e);
                    }

                }
            }
        };

        crate::utils::trace_ref::<dyn IUnknown>(&instance.instance);

        Ok(Self {
            controller,
            component_handler,
            is_instance
        })
    }

    pub fn initialize(&mut self, host: &Host) -> Result<(), Error> {

        if !self.is_instance {
            return Ok(());
        }

        let host_context = host.get_context()?;
        let host_context_ptr = host_context.as_ptr();
        let result = unsafe {
            if ENABLE_COMPONENT_HANDLER {
                self.controller.set_component_handler(self.component_handler.get_shared_ptr());
            }

            self.controller.initialize(host_context_ptr as *mut c_void)
        };
        if result != kResultOk {
            return Err(Error::from("failed to initialize edit controller component"));
        }

        Ok(())
    }

    pub fn terminate(&mut self) -> Result<(), Error> {
        if self.is_instance {
            unsafe {
                self.controller.terminate();
            }
        }

        Ok(())
    }

    pub fn dispose(&mut self) {
        let _ = self.terminate();
        self.is_instance = false;
    }

    pub fn set_component_state(&self, state: &mut ByteStream) -> Result <(), Error> {
        trace!("set component state");
        let state_intf = state.get_shared_ptr();
        let result = unsafe { self.controller.set_component_state(state_intf) };
        if result != kResultOk {
            return Err(Error::from("failed to get edit controller state"));
        }
        Ok(())
    }

    pub fn set_state(&self, state: &mut ByteStream) -> Result <(), Error> {
        trace!("set state");

        let state_intf = state.get_shared_ptr();
        let result = unsafe { self.controller.set_state(state_intf) };
        if result != kResultOk {
            return Err(Error::from("failed to get edit controller state"));
        }
        Ok(())
    }

    pub fn get_state(&self, state: &mut ByteStream) -> Result <(), Error> {
        trace!("get state");

        let state_intf = state.get_shared_ptr();
        let result = unsafe { self.controller.get_state(state_intf) };
        if result != kResultOk {
            return Err(Error::from("failed to get edit controller state"));
        }
        Ok(())
    }

    pub fn get_parameter_count(&self) -> i32 {
        trace!("get parameter count");

        unsafe { self.controller.get_parameter_count() }
    }

    pub fn get_parameter_info(&self, param_index: i32) -> Result<ParameterInfo, Error> {
        trace!("get parameter info");

        let mut param_info = ParameterInfo {
            id: 0,
            title: [0; 128],
            short_title: [0; 128],
            units: [0; 128],
            step_count: 0,
            default_normalized_value: 0.0,
            unit_id: 0,
            flags: 0
        };

        let result = unsafe {
            self.controller.get_parameter_info(param_index, &mut param_info)
        };

        if result != kResultOk {
            return Err(Error::from("failed to get edit parameter info"));
        }

        Ok(param_info)
    }

    pub fn get_param_string_by_value(&self, id: u32, value_normalized: f64) -> Result<String, Error> {
        trace!("get param string by value");

        let mut str_buffer: String128 = [0; 128];
        let ptr_str_buffer = str_buffer.as_mut_ptr();
        let result = unsafe {
            self.controller.get_param_string_by_value(id, value_normalized, ptr_str_buffer)
        };

        if result != kResultOk {
            return Err(Error::from("failed to get param string by value"));
        }

        let value = String::from_utf16(unsafe { std::slice::from_raw_parts(ptr_str_buffer as *mut u16, 128) }).unwrap();

        Ok(value)

    }

    pub fn get_param_value_by_string(&self, id: u32, s: &str) -> Result<f64, Error> {
        trace!("get param value by string");

        let s128: Vec<u16> = s.encode_utf16().collect();
        let mut str_buffer: String128 = [0; 128];

        let mut idx = 0;
        for c in s128 {
            if idx >= 127 { break }
            str_buffer[idx] = c as i16;
            idx += 1;
        }

        let mut value_normalized: f64 = 0.0;

        let result = unsafe {
            self.controller.get_param_value_by_string(id, str_buffer.as_ptr(), &mut value_normalized)
        };

        if result != kResultOk {
            return Err(Error::from("failed to get param value by string"));
        }

        Ok(value_normalized)
    }

    pub fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        trace!("normalized param to plain");
        unsafe { self.controller.normalized_param_to_plain(id, value_normalized) }
    }

    pub fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        trace!("plain param to normalized");
        unsafe { self.controller.plain_param_to_normalized(id, plain_value) }
    }

    pub fn get_param_normalized(&self, id: u32) -> f64 {
        trace!("get param normalized");
        unsafe { self.controller.get_param_normalized(id) }
    }

    pub fn set_param_normalized(&self, id: u32, value: f64) -> Result<(), Error> {
        trace!("set param normalized");

        let result = unsafe { self.controller.set_param_normalized(id, value) };

        if result != kResultOk {
            return Err(Error::from("failed to set component handler"));
        }

        Ok(())
    }

    pub fn set_component_handler(&self, handler: SharedVstPtr<dyn IComponentHandler>) -> Result<(), Error> {

        trace!("set component handler");

        let result = unsafe { self.controller.set_component_handler(handler) };

        if result != kResultOk {
            return Err(Error::from("failed to set component handler"));
        }

        Ok(())
    }

    pub fn create_view(&self) -> Result<View, Error> {

        trace!("create_view");

        let view_type = "editor\0";
        let view_type_ptr = view_type.as_ptr();

        let intf_ptr  = unsafe {
            self.controller.create_view(view_type_ptr as *const i8)
        };

        if intf_ptr.is_null() {
            return Err(Error::from("failed to create view"));
        }

        let plugview = match unsafe { VstPtr::<dyn IPlugView>::owned(intf_ptr as *mut *mut IPlugViewVTable) } {
            Some(unk_intf_ptr) => unk_intf_ptr,
            None => {
                return Err(Error::from("failed to get plug view interface"));
            }
        };

        let view = View::new(plugview)?;

        Ok(view)

    }

}

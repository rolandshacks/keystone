use log::{*};
use std::sync::Arc;
use vst3_com::{sys::GUID, *};
use vst3_sys::{base::*, vst::{IAudioProcessor, IComponent, IEditController, IProcessContextRequirements, IoModes}};

use crate::{error::Error, host::Host, plugin::Plugin};

pub struct Instance {
    pub class_id: String,
    pub plugin: Arc<Plugin>,
    pub instance: VstPtr<dyn IUnknown>,
    pub component: VstPtr<dyn IComponent>
}

impl Drop for Instance {
    fn drop(&mut self) {
        trace!("drop Instance");
    }
}

impl Instance {
    pub fn new(plugin: &Arc<Plugin>, instance: VstPtr<dyn IUnknown>, class_id: &str) -> Result<Self, Error> {
        trace!("new");

        let component = Self::query_component_intf(&instance)?;

        Ok(Self {
            class_id: class_id.to_string(),
            plugin: plugin.clone(),
            instance,
            component
        })
    }

    pub fn dispose(&mut self) {
    }

    pub fn create_class_instance(&self, cid: &GUID) -> Result<VstPtr<dyn IUnknown>, Error> {
        trace!("create class instance");

        if self.plugin.lib.is_none() {
            return Err(Error::from("plugin not loaded"));
        }

        self.plugin.create_class_instance(&cid)
    }

    pub fn class_id(&self) -> &str {
        &self.class_id
    }

    pub fn get_plugin(&self) -> Arc<Plugin> {
        self.plugin.clone()
    }

    pub fn initialize(&self, host: &Host) -> Result<(), Error> {
        let host_context = host.get_context()?;
        let host_context_ptr = host_context.as_ptr();
        let result = unsafe { self.component.initialize(host_context_ptr as *mut c_void) };
        if result != kResultOk {
            return Err(Error::from("failed to initialize component"));
        }
        Ok(())
    }

    pub fn terminate(&self) -> Result<(), Error> {
        let result = unsafe { self.component.terminate() };
        if result != kResultOk {
            return Err(Error::from("failed to terminate component"));
        }
        Ok(())
    }

    pub fn get_controller_class_id(&self) -> Result<GUID, Error> {

        let mut controller_guid = GUID {
            data: [0u8; 16]
        };

        let result = unsafe { self.component.get_controller_class_id(&mut controller_guid) };
        if result != kResultOk {
            // no controller class id
            return Err(Error::from("failed to get controller class id"));
        }

        Ok(controller_guid)
    }

    pub fn set_active(&self, active: bool) -> Result<(), Error> {
        let result = unsafe { self.component.set_active(if active { 1 } else { 0 }) };
        if result != kResultOk {
            return Err(Error::from("failed to activate component"))
        }
        Ok(())
    }

    pub fn set_io_mode(&mut self, mode: IoModes) {
        unsafe {
            self.component.set_io_mode(mode as i32);
        }
    }

    fn query_component_intf(instance: &VstPtr<dyn IUnknown>) -> Result<VstPtr<dyn IComponent>, Error> {
        let component = match instance.cast::<dyn IComponent>() {
            Some(component_intf) => {
                component_intf
            }
            None => {
                return Err(Error::from("failed to query component interface"));
            }
        };

        Ok(component)
    }

    pub fn query_audio_processor_intf(&self) -> Result<VstPtr<dyn IAudioProcessor>, Error> {
        let intf = match self.instance.cast::<dyn IAudioProcessor>() {
            Some(intf) => intf,
            None => {
                return Err(Error::from("failed to query audio processor interface"));
            }
        };

        Ok(intf)
    }

    pub fn create_edit_controller_intf(&self) -> Result<VstPtr<dyn IEditController>, Error> {
        // separate edit controller instance
        let intf = match self.get_controller_class_id() {
            Ok(guid) => {
                let iunk = self.create_class_instance(&guid)?;
                match iunk.cast::<dyn IEditController>() {
                    Some(intf) => {
                        trace!("created new edit controller instance");
                        intf
                    },
                    None => {
                        return Err(Error::from("failed to get controller interface"));
                    }
                }
            },
            Err(_) => {
                return Err(Error::from("failed to get controller class id"))
            }
        };

        Ok(intf)
    }

    pub fn query_edit_controller_intf(&self) -> Result<VstPtr<dyn IEditController>, Error> {

        let intf = match self.instance.cast::<dyn IEditController>() {
            Some(intf) => {
                trace!("queried edit controller from instance");
                // instance already implements edit controller
                intf
            },
            None => {
                return Err(Error::from("instance does not provide edit controller interface"));
            }
        };

        Ok(intf)
    }

    pub fn query_process_context_requirements_intf(&self) -> Result<VstPtr<dyn IProcessContextRequirements>, Error> {
        let intf = match self.instance.cast::<dyn IProcessContextRequirements>() {
            Some(intf) => intf,
            None => {
                return Err(Error::from("failed to query process context requirements interface"));
            }
        };

        unsafe {
            intf.add_ref();
        }

        Ok(intf)
    }

}

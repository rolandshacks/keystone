use std::{ffi::c_void, ptr::null_mut};

use log::{*};
use vst3_com::{interfaces::iunknown::IID_IUNKNOWN, VstPtr, IID, REFIID};
use vst3_sys::{base::{kResultFalse, kResultOk, tresult, IUnknown}, utils::StaticVstPtr, vst::{IHostApplication, IHostApplicationVTable, IPlugInterfaceSupport}, VST3};

use crate::error::Error;

#[VST3(implements(IHostApplication, IPlugInterfaceSupport))]
pub struct HostApplication {
}

impl HostApplication {
    pub fn new() -> Box<Self> {
        let instance = Self::allocate();
        instance
    }

    pub fn get_static_ptr(&mut self) -> StaticVstPtr<dyn IHostApplication> {
        let static_vst_ptr: StaticVstPtr<dyn IHostApplication> = unsafe {
            std::mem::transmute(self as *mut _)
        };
        static_vst_ptr
    }

    pub fn get_null_ptr() -> StaticVstPtr<dyn IHostApplication> {
        let null_ptr: *mut IHostApplicationVTable = null_mut();
        let ptr: StaticVstPtr<dyn IHostApplication> = unsafe {
            std::mem::transmute(null_ptr as *mut _)
        };
        return ptr
    }

    pub fn get_context(&self) -> Result<VstPtr<dyn IUnknown>, Error> {
        let mut ppv_host_context: *mut c_void = null_mut();
        let host_context: VstPtr<dyn IUnknown> = unsafe {
            match self.query_interface(&IID_IUNKNOWN, &mut ppv_host_context as *mut *mut c_void) {
                kResultOk => {
                    VstPtr::owned(ppv_host_context as *mut *mut _).unwrap()
                },
                _ => {
                    return Err(Error::from("unable to get host context"));
                }
            }
        };

        Ok(host_context)
    }

}

impl IHostApplication for HostApplication {
    unsafe fn get_name(&self, name: *mut u16) -> tresult {

        let name_str = "Keystone";
        let name_utf16: Vec<u16> = name_str.encode_utf16().collect();
        let name_slice = std::slice::from_raw_parts_mut(name, 128);

        let mut idx = 0;
        for c in name_utf16 {
            if idx >= 127 { break }
            name_slice[idx] = c;
            idx += 1;
        }

        name_slice[idx] = 0;

        kResultOk
    }

    unsafe fn create_instance(&self, _cid: *const IID, _iid: *const IID, _obj: *mut *mut c_void) -> tresult {
        kResultFalse
    }

}

impl IPlugInterfaceSupport for HostApplication {
    unsafe fn is_pluginterface_supported(&self, _iid: REFIID) -> tresult {
        kResultFalse
    }
}

pub struct Host {
    host_application: Box<HostApplication>
}

impl Host {
    pub fn new() -> Result<Self, Error> {
        trace!("new");
        let host_application = HostApplication::new();
        Ok(Self {
            host_application
        })
    }

    pub fn dispose(&mut self) {
        trace!("dispose");
    }

    pub fn get_context(&self) -> Result<VstPtr<dyn IUnknown>, Error> {
        self.host_application.get_context()
    }
}

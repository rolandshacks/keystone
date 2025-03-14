use std::ptr::null_mut;

use vst3_sys::{base::{kResultFalse, tresult}, utils::StaticVstPtr, vst::{IParamValueQueue, IParamValueQueueVTable, IParameterChanges, IParameterChangesVTable}, VST3};


#[VST3(implements(IParamValueQueue))]
pub struct ParamValueQueue {
}

impl ParamValueQueue {
    pub fn new() -> Box<Self> {
        let instance = Self::allocate();
        instance
    }

    pub fn get_static_ptr(&self) -> StaticVstPtr<dyn IParamValueQueue> {
        let static_vst_ptr: StaticVstPtr<dyn IParamValueQueue> = unsafe {
            std::mem::transmute(self as *const _)
        };
        static_vst_ptr
    }

    pub fn get_null_ptr() -> StaticVstPtr<dyn IParamValueQueue> {
        let null_ptr: *mut IParamValueQueueVTable = null_mut();
        let ptr: StaticVstPtr<dyn IParamValueQueue> = unsafe {
            std::mem::transmute(null_ptr as *mut _)
        };
        return ptr
    }
}

impl IParamValueQueue for ParamValueQueue {
    unsafe fn get_parameter_id(&self) -> u32 {
        0
    }

    unsafe fn get_point_count(&self) -> i32 {
        0
    }

    unsafe fn get_point(&self, _index: i32, _sample_offset: *mut i32, _value: *mut f64) -> tresult {
        kResultFalse
    }

    unsafe fn add_point(&self, _sample_offset: i32, _value: f64, _index: *mut i32) -> tresult {
        kResultFalse
    }
}

#[VST3(implements(IParameterChanges))]

pub struct ParameterChanges {
    param_value_queue: Box<ParamValueQueue>
}

impl ParameterChanges {
    pub fn new() -> Box<Self> {
        let param_value_queue = ParamValueQueue::new();
        let instance = Self::allocate(param_value_queue);
        instance
    }

    pub fn get_static_ptr(&mut self) -> StaticVstPtr<dyn IParameterChanges> {
        let static_vst_ptr: StaticVstPtr<dyn IParameterChanges> = unsafe {
            std::mem::transmute(self as *mut _)
        };
        static_vst_ptr
    }

    pub fn get_null_ptr() -> StaticVstPtr<dyn IParameterChanges> {
        let null_ptr: *mut IParameterChangesVTable = null_mut();
        let ptr: StaticVstPtr<dyn IParameterChanges> = unsafe {
            std::mem::transmute(null_ptr as *mut _)
        };
        return ptr
    }

}

impl IParameterChanges for ParameterChanges {
    unsafe fn get_parameter_count(&self) -> i32 {
        0
    }

    unsafe fn get_parameter_data(&self, index: i32) -> StaticVstPtr<dyn IParamValueQueue>  {
        if index != 0 {
            return ParamValueQueue::get_null_ptr();
        }

        ParamValueQueue::get_static_ptr(self.param_value_queue.as_ref())
    }

    unsafe fn add_parameter_data(&self, _id: *const u32, index: *mut i32,) -> StaticVstPtr<dyn IParamValueQueue>  {
        if !index.is_null() {
            *index = 0;
        }

        ParamValueQueue::get_static_ptr(self.param_value_queue.as_ref())
    }
}

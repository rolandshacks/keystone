
use log::{*};
use std::ptr::null_mut;

use vst3_sys::{base::{kResultFalse, tresult, IBStream, IBStreamVTable}, utils::SharedVstPtr, VST3};


#[VST3(implements(IBStream))]
pub struct ByteStream {
}

impl ByteStream {
    pub fn new() -> Box<Self> {
        let instance = Self::allocate();
        instance
    }

    pub fn get_shared_ptr(&mut self) -> SharedVstPtr<dyn IBStream> {
        let shared_vst_ptr: SharedVstPtr<dyn IBStream> = unsafe {
            std::mem::transmute(self as * mut _)
        };
        shared_vst_ptr
    }

    pub fn get_null_ptr() -> SharedVstPtr<dyn IBStream> {
        let null_ptr: *mut IBStreamVTable = null_mut();
        let ptr: SharedVstPtr<dyn IBStream> = unsafe {
            std::mem::transmute(null_ptr as *mut _)
        };
        return ptr
    }
}

impl IBStream for ByteStream {
    unsafe fn read(&self, _buffer: *mut std::ffi::c_void, num_bytes: i32, _num_bytes_read: *mut i32) -> tresult {
        trace!("stream::read {} bytes", num_bytes);
        kResultFalse
    }

    unsafe fn write(&self, _buffer: *const std::ffi::c_void, num_bytes: i32, _num_bytes_written: *mut i32,) -> tresult {
        trace!("stream::write {} bytes", num_bytes);
        kResultFalse
    }

    unsafe fn seek(&self, pos: i64, mode: i32, _result: *mut i64) -> tresult {
        trace!("stream::seek pos:{}, mode:{}", pos, mode);
        kResultFalse
    }

    unsafe fn tell(&self, _pos: *mut i64) -> tresult {
        trace!("stream::tell");
        kResultFalse
    }
}

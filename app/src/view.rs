use std::{cell::RefCell, ffi::c_void, ptr::null_mut};

use log::{*};
use vst3_com::VstPtr;
use vst3_sys::{base::{kResultOk, kResultTrue, tresult, IUnknown}, gui::{IPlugFrame, IPlugFrameVTable, IPlugView, IPlugViewContentScaleSupport, ViewRect}, utils::SharedVstPtr, VST3};
use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::PostMessageA};

use crate::{constants::WM_USER_VIEW_RESIZE, error::Error, utils::Size};

#[VST3(implements(IPlugFrame))]
pub struct PlugFrame {
    hwnd: HWND,
    recursion_guard: RefCell<bool>
}

impl Drop for PlugFrame {
    fn drop(&mut self) {
        trace!("drop PlugFrame");
    }
}

impl PlugFrame {
    pub fn new() -> Box<Self> {
        let instance = Self::allocate(null_mut(), RefCell::new(false));
        instance
    }

    pub fn get_shared_ptr(&mut self) -> SharedVstPtr<dyn IPlugFrame> {
        let shared_vst_ptr: SharedVstPtr<dyn IPlugFrame> = unsafe {
            std::mem::transmute(self as * mut _)
        };
        shared_vst_ptr
    }

    pub fn get_null_ptr() -> SharedVstPtr<dyn IPlugFrame> {
        let null_ptr: *mut IPlugFrameVTable = null_mut();
        let ptr: SharedVstPtr<dyn IPlugFrame> = unsafe {
            std::mem::transmute(null_ptr as *mut _)
        };
        return ptr
    }

    pub fn attach(&mut self, hwnd: HWND) {
        self.hwnd = hwnd;
    }

    pub fn detach(&mut self) {
        self.hwnd = null_mut();
    }

}

impl IPlugFrame for PlugFrame {
    unsafe fn resize_view(&self, view_ptr: SharedVstPtr<dyn IPlugView>, _new_size: *mut ViewRect) -> tresult {
        trace!("resize view");

        if self.recursion_guard.replace(true) {
            return kResultOk;
        }

        self.recursion_guard.replace(true);

        let view = view_ptr.upgrade().unwrap();
        let mut size_rect = ViewRect::default();
        view.get_size(&mut size_rect);
        //trace!("resize view {:?}", size_rect);

        let width = size_rect.right - size_rect.left;
        let height = size_rect.bottom - size_rect.top;

        if !self.hwnd.is_null() {
            unsafe {
                PostMessageA(self.hwnd, WM_USER_VIEW_RESIZE, width as usize, height as isize);
            }
        }

        self.recursion_guard.replace(false);

        kResultOk
    }
}

pub struct View {
    plug_view: VstPtr<dyn IPlugView>,
    plug_frame: Box<PlugFrame>
}

impl Drop for View {
    fn drop(&mut self) {
        trace!("drop View");
    }
}

impl View {
    pub fn new(plug_view: VstPtr<dyn IPlugView>) -> Result<Self, Error> {
        trace!("new");

        let mut plug_frame = PlugFrame::new();

        unsafe {
            let frame_ptr = plug_frame.get_shared_ptr().as_ptr();
            plug_view.set_frame(frame_ptr as *mut _);
        };

        let view = Self {
            plug_view,
            plug_frame
        };

        Ok(view)
    }

    pub fn release(&mut self) {
        trace!("release View");
        unsafe {
            self.plug_view.set_frame(null_mut());
            // actual release of plug frame when dropping

            self.plug_view.add_ref();
            let ref_count = self.plug_view.release();
            trace!("plug view release {}", ref_count);
            assert!(ref_count == 1);
            // actual release takes place when VstPtr<IPlugView> gets dropped
        }
    }

    pub fn attach(&mut self, hwnd: *mut c_void) -> Result<(), Error> {
        trace!("attached");
        let view_type = "HWND\0";
        let view_type_ptr = view_type.as_ptr();
        let result = unsafe { self.plug_view.attached(hwnd, view_type_ptr as *const i8) };
        if result != kResultOk {
            return Err(Error::from("failed to attach view to window"));
        }

        self.plug_frame.attach(hwnd);

        Ok(())
    }

    pub fn detach(&mut self) -> Result<(), Error> {
        trace!("removed");

        self.plug_frame.detach();

        let result = unsafe { self.plug_view.removed() };
        if result != kResultOk {
            return Err(Error::from("failed to remove view to window"));
        }

        Ok(())
    }

    pub fn can_resize(&self) -> bool {
        unsafe { self.plug_view.can_resize() == kResultTrue }
    }

    pub fn get_size(&self) -> Size {
        trace!("get size");
        let mut plug_view_size = ViewRect::default();
        let _ = unsafe { self.plug_view.get_size(&mut plug_view_size) };
        Size::new(plug_view_size.right, plug_view_size.bottom)
    }

    pub fn on_size(&self, width: i32, height: i32) -> Result<(), Error> {
        trace!("on size({}, {})", width, height);

        let mut new_size = ViewRect {
            left: 0,
            top: 0,
            right: width,
            bottom: height
        };

        unsafe {

            let mut old_size = ViewRect::default();
            if self.plug_view.get_size(&mut old_size) == kResultTrue {
                if is_equal(&old_size, &new_size) {
                    return Ok(());
                }
            }

            self.plug_view.on_size(&mut new_size);
        };

        Ok(())
    }

    pub fn constrain_size(&self, size: &Size) -> Option<Size> {
        unsafe {
            let mut r = ViewRect {
                left: 0,
                top: 0,
                right: size.width,
                bottom: size.height
            };

            if self.plug_view.check_size_constraint(&mut r) != kResultTrue {
                self.plug_view.get_size(&mut r);
                let new_size = Size::new(r.right - r.left, r.bottom - r.top);
                if new_size != *size {
                    return Some(new_size);
                }
            }
        }

        None
    }

    pub fn on_content_scale_factor_changed(&mut self, factor: f64) {

        match self.plug_view.cast::<dyn IPlugViewContentScaleSupport>() {
            Some(intf) => {
                unsafe { intf.set_scale_factor(factor as f32) };
            },
            None => {}
        };

    }

}

fn is_equal(a: &ViewRect, b: &ViewRect) -> bool {
    a.left == b.left &&
    a.top == b.top &&
    a.right == b.right &&
    a.bottom == b.bottom
}

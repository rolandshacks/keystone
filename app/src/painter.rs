//!
//! Painter
//!

use std::{ffi::CString, ptr::null_mut};

use log::{*};
use windows_sys::Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*};

#[repr(C)]
pub struct PaintObject(*mut core::ffi::c_void);

impl Drop for PaintObject {
    fn drop(&mut self) {
        self.delete();
    }
}

impl PaintObject {
    pub fn from(hobject: *mut core::ffi::c_void) -> Self {
        Self {
            0: hobject
        }
    }

    pub fn null() -> Self {
        Self {
            0: null_mut()
        }
    }

    pub fn delete(&mut self) {
        if !self.0.is_null() {
            let hobject = self.0;
            self.0 = null_mut();

            unsafe {
                DeleteObject(hobject);
            }
        }
    }

    pub fn as_ptr(&self) -> *mut core::ffi::c_void {
        self.0
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

}

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (((b as u32) << 16) | ((g as u32) << 8) | (r as u32)) as u32
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (((a as u32) << 24) |((b as u32) << 16) | ((g as u32) << 8) | (r as u32)) as u32
}

pub struct Painter {
    hwnd: HWND,
    ps: PAINTSTRUCT,
    hdc: HDC,
    background: PaintObject
}

impl Painter {
    pub fn new() -> Self {
        let ps: PAINTSTRUCT = unsafe { core::mem::zeroed() };
        Self {
            hwnd: null_mut(),
            ps,
            hdc: null_mut(),
            background: PaintObject::null()
        }
    }

    pub fn dispose(&mut self) {
        if !self.hwnd.is_null() {
            self.background.delete();
            self.hwnd = null_mut();
        }
    }

    pub fn begin(&mut self, hwnd: HWND) {
        trace!("begin");
        unsafe {
            let hdc = BeginPaint(hwnd, &mut self.ps);

            if self.background.is_null() {
                self.background = PaintObject::from(CreateSolidBrush(rgb(32, 32, 32)));
            }

            self.hdc = hdc;
        }
        self.hwnd = hwnd;
    }

    pub fn end(&mut self) {
        trace!("end");
        unsafe {
            EndPaint(self.hwnd, &mut self.ps);
        }
        //unsafe { ValidateRect(hwnd, std::ptr::null()); }
        self.hdc = null_mut();
    }

    pub fn draw_text(&self, text: &str, x: i32, y: i32) {
        let hdc = self.hdc;
        assert!(!hdc.is_null());

        unsafe {
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, rgb(0xff, 0xff, 0xff));
            let s = CString::new(text).unwrap();
            TextOutA(hdc, x, y, s.as_ptr() as *const u8, text.len() as i32);
        }
    }

    pub fn clear(&self) {
        let hwnd = self.hwnd;
        let hdc = self.hdc;

        unsafe {
            let mut r: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut r);
            FillRect(hdc, &r, self.background.as_ptr());
        }
    }

}

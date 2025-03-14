
use std::{ffi::c_void, ptr::null_mut, time::Duration};

use log::{*};
use windows_sys::{
    core::*, Win32::{Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleA, UI::WindowsAndMessaging::*, UI::HiDpi::*},
};

use crate::{config::ENABLE_VIEW_RESIZE, constants::WM_USER_VIEW_RESIZE, error::Error, painter::Painter, utils::Size, view::View};


pub struct Window {
    instance: *mut c_void,
    hwnd: *mut c_void,
    view: Option<View>,
    dpi_changing: bool,
    dpi_changed_size: Size,
    painter: Painter
}

impl Drop for Window {
    fn drop(&mut self) {
        trace!("drop Window");
    }
}

impl Window {
    pub fn new(title: &str, width: i32, height: i32, resizeable: bool) -> Result<Box<Self>, Error> {
        trace!("new window");

        let instance = unsafe { GetModuleHandleA(std::ptr::null()) };
        debug_assert!(!instance.is_null());

        let mut window = Box::new(Self {
            instance,
            hwnd: null_mut(),
            view: None,
            dpi_changing: false,
            dpi_changed_size: Size::default(),
            painter: Painter::new()
        });

        let window_ptr = &mut *window as *mut _;  //&mut window as *mut Window;
        let window_class = s!("window");
        let cursor = unsafe { LoadCursorW(core::ptr::null_mut(), IDC_ARROW) };
        let window_title = std::ffi::CString::new(title).unwrap();

        let wc = WNDCLASSA {
            hCursor: cursor,
            hInstance: instance,
            lpszClassName: window_class,
            style: CS_DBLCLKS, // CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: core::ptr::null_mut(),
            hbrBackground: core::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
        };

        let atom = unsafe { RegisterClassA(&wc) };
        debug_assert!(atom != 0);

        let ex_style = WS_EX_APPWINDOW;
        let mut dw_style = WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN | WS_CLIPSIBLINGS;
        if resizeable {
            dw_style |= WS_SIZEBOX | WS_MAXIMIZEBOX;
        }

        let hwnd = unsafe {

            let mut rect = RECT { left: 0, top: 0, right: width, bottom: height };
            AdjustWindowRectEx(&mut rect, dw_style, 0, ex_style);

            CreateWindowExA(
                ex_style,
                window_class,
                window_title.as_ptr() as *const u8, //s!(title),
                dw_style,
                2400 - rect.right, // CW_USEDEFAULT,
                50, // CW_USEDEFAULT,
                rect.right - rect.left, // CW_USEDEFAULT,
                rect.bottom - rect.top, // CW_USEDEFAULT,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                instance,
                window_ptr as *mut c_void,
            )
        };

        debug!("HWND (from CreateWindowEx): {:016p}", hwnd);

        if hwnd.is_null() {
            return Err(Error::from("failed to create window"));
        }

        if window.hwnd.is_null() {
            window.hwnd = hwnd;
        }

        Ok(window)

    }

    pub fn dispose(&mut self) {
        let _ = self.stop_timer();
        let _ = self.detach_view();
        if !self.hwnd.is_null() {
            self.painter.dispose();
            unsafe { CloseWindow(self.hwnd) };
            self.hwnd = null_mut();
        }
    }

    pub fn handle(&self) -> *mut c_void {
        self.hwnd
    }

    pub fn show(&mut self) {
        trace!("show");

        let hwnd = self.hwnd;

        self.on_scale_factor_changed(Self::get_content_scale_factor(hwnd));

        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            SetWindowPos (hwnd, HWND_TOP, 0, 0, 0, 0,
                SWP_NOSIZE | SWP_NOMOVE | SWP_NOCOPYBITS | SWP_SHOWWINDOW);
        }
    }

    pub fn hide(&mut self) {
        trace!("hide");
        unsafe { ShowWindow(self.hwnd, SW_HIDE); }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        trace!("resize");
        if self.dpi_changing {
            self.dpi_changed_size.set(width, height);
            return;
        }

        let new_size = Size::new(width, height);
        let current_size = self.get_content_size();
        if new_size.equals(&current_size) {
            return;
        }

        unsafe {
            let hwnd = self.hwnd;
            let mut window_info: WINDOWINFO = std::mem::zeroed();
            GetWindowInfo(hwnd, &mut window_info);
            let mut client_rect: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rect);
            client_rect.right = new_size.width;
            client_rect.bottom = new_size.height;
            AdjustWindowRectEx(&mut client_rect, window_info.dwStyle, 0, window_info.dwExStyle);
            SetWindowPos(hwnd, HWND_TOP, 0, 0, client_rect.right - client_rect.left, client_rect.bottom - client_rect.top, SWP_NOMOVE | SWP_NOCOPYBITS | SWP_NOACTIVATE);
            InvalidateRect(hwnd, null_mut(), 0);
        }

    }

    pub fn start_timer(&mut self, delay: &Duration) -> Result<(), Error> {
        if self.hwnd.is_null() {
            return Err(Error::from("invalid window handle"));
        }

        unsafe { SetTimer(self.hwnd, 1, delay.as_millis() as u32, None) };

        Ok(())
    }

    pub fn stop_timer(&mut self) -> Result<(), Error> {
        if self.hwnd.is_null() {
            return Err(Error::from("invalid window handle"));
        }

        unsafe { KillTimer(self.hwnd, 1) };

        Ok(())
    }

    pub fn event_loop(&self) {
        trace!("event loop");
        unsafe {
            let mut message = std::mem::zeroed();
            while GetMessageA(&mut message, core::ptr::null_mut(), 0, 0) != 0 {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }
        }
    }

    fn on_create(&mut self) {
        trace!("on create");
    }

    fn on_close(&mut self) {
        let _ = self.detach_view();
    }

    fn on_destroy(&mut self) {
        trace!("on destroy");
    }

    fn on_paint(&mut self) {
        //trace!("on paint");
        self.painter.clear();
        self.painter.draw_text("Keystone", 20, 20);
    }

    fn on_resize(&mut self, width: i32, height: i32) {
        trace!("on resize ({},{})", width, height);

        if self.view.is_some() {
            let view = self.view.as_mut().unwrap();
            let _ = view.on_size(width, height);
        }
    }

    fn on_view_resize(&mut self, width: i32, height: i32) {
        trace!("on view resize ({},{})", width, height);
        self.resize(width, height);
    }

    fn on_scale_factor_changed(&mut self, factor: f64) {
        trace!("on scale factor changed ({})", factor);

        if self.view.is_some() {
            let view = self.view.as_mut().unwrap();
            let _ = view.on_content_scale_factor_changed(factor);
        }

    }

    fn on_timer(&mut self) {
        trace!("on timer");
    }

    pub fn on_event(&mut self, hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {

        if self.hwnd.is_null() {
            self.hwnd = hwnd;
        }

        let mut result: LRESULT = 0;

        let mut handle_default = false;

        match message {
            WM_CREATE => {
                self.on_create();
                handle_default = true;
            }
            WM_ERASEBKGND => {
                result = 1;
            }
            WM_PAINT => {
                self.painter.begin(hwnd);
                self.on_paint();
                self.painter.end();
            }
            WM_SIZE => {
                //let width = lo_word(lparam as u32) as i32;
                //let height = hi_word(lparam as u32) as i32;
                let size = Self::get_client_size(hwnd);
                self.on_resize(size.width, size.height);
            }
            WM_SIZING => {

                let new_rect = lparam as *mut RECT;
                let new_size = unsafe { Size::new((*new_rect).right - (*new_rect).left, (*new_rect).bottom - (*new_rect).top) };

                let old_size = unsafe {
                    let mut r: RECT = std::mem::zeroed();
                    GetWindowRect(hwnd, &mut r);
                    Size::new(r.right - r.left, r.bottom - r.top)
                };

                let diff = Size::diff(&old_size, &new_size);
                let mut client_size = Self::get_client_size(hwnd);
                client_size.add_size(&diff);

                if self.view.is_some() {
                    match self.view.as_ref().unwrap().constrain_size(&new_size) {
                        Some(constraint_size) => {
                            let diff2 = Size::diff(&old_size, &client_size);
                            unsafe {
                                (*new_rect).right = (*new_rect).left + constraint_size.width + diff2.width;
                                (*new_rect).bottom = (*new_rect).top + constraint_size.height + diff2.height;                            }
                        },
                        None => {}
                    };
                }

                result = 1;
            }
            WM_USER_VIEW_RESIZE => {
                let width = wparam as i32;
                let height = lparam as i32;
                self.on_view_resize(width, height);
            }
            WM_GETDPISCALEDSIZE => {
                self.dpi_changing = true;

                let dpi = wparam as f64;
                let default_screen_dpi = USER_DEFAULT_SCREEN_DPI as f64;
                let new_scale_factor = if default_screen_dpi > 0.0 { dpi / default_screen_dpi } else { 1.0 };
                self.on_scale_factor_changed(new_scale_factor);


                if self.dpi_changed_size.width != 0 && self.dpi_changed_size.height != 0 {

                    unsafe {
                        let mut window_info: WINDOWINFO = std::mem::zeroed();
                        GetWindowInfo(hwnd, &mut window_info);

                        let mut client_rect: RECT = std::mem::zeroed();
                        client_rect.right = self.dpi_changed_size.width;
                        client_rect.bottom = self.dpi_changed_size.height;

                        AdjustWindowRectExForDpi(
                            &mut client_rect,
                            window_info.dwStyle,
                            0,
                            window_info.dwExStyle,
                            wparam as u32
                        );

                        let mut proposed_size: SIZE = std::mem::zeroed();
                        proposed_size.cx = client_rect.right - client_rect.left;
                        proposed_size.cy = client_rect.bottom - client_rect.top;

                        return 1;
                    }
                }

            }
            WM_DPICHANGED => {
                if self.dpi_changing {
                    self.dpi_changing = false;
                    self.dpi_changed_size.zero();

                    let r = lparam as *mut RECT;
                    unsafe {
                        SetWindowPos(
                            hwnd,
                            null_mut(),
                            (*r).left, (*r).top,
                            (*r).right - (*r).left,
                            (*r).bottom - (*r).top,
                            SWP_NOZORDER | SWP_NOACTIVATE);
                    }
                } else {
                    self.on_scale_factor_changed(Self::get_content_scale_factor(hwnd));
                }
            }
            WM_CLOSE => {
                self.on_close();
                unsafe { DestroyWindow(hwnd) };
            }
            WM_DESTROY => {
                unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0 as isize); }
                self.on_destroy();
                unsafe { PostQuitMessage(0); }
            }
            WM_TIMER => {
                self.on_timer();
            }
            _ => {
                handle_default = true;
            }
        }

        if handle_default {
            result = unsafe { DefWindowProcA(hwnd, message, wparam, lparam) }
        }

        result
    }


    fn get_dpi_for_window(hwnd: HWND) -> (u32, u32) {
        unsafe {
    		let monitor = MonitorFromWindow (hwnd, MONITOR_DEFAULTTONEAREST);
            let mut x: u32 = 0;
            let mut y: u32 = 0;
            let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);
            (x, y)
        }
    }

    fn get_client_size(hwnd: HWND) -> Size {
        unsafe {
            let mut r: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut r);
            Size::new(r.right - r.left, r.bottom - r.top)
        }
    }

    pub fn get_content_size(&self) -> Size {
        Self::get_client_size(self.hwnd)
    }

    fn get_content_scale_factor(hwnd: HWND) -> f64 {
        let dpi = Self::get_dpi_for_window(hwnd);
        let default_screen_dpi = USER_DEFAULT_SCREEN_DPI as f64;
        let factor = if default_screen_dpi > 0.0 { (dpi.0 as f64) / default_screen_dpi } else { 1.0 };
        factor
    }

    pub fn attach_view(&mut self, mut view: View) -> Result<(), Error> {
        self.detach_view()?;

        if self.hwnd.is_null() {
            panic!("HWND expected to be not null!");
        }

        view.attach(self.handle())?;

        if ENABLE_VIEW_RESIZE && view.can_resize() {
            // resize view to window
            let size = Self::get_client_size(self.hwnd);
            let _ = view.on_size(size.width, size.height);
        } else {
            // resize window to view
            let sz = view.get_size();
            if sz.width > 16 && sz.height > 16 {
                self.resize(sz.width, sz.height);
            }
        }

        self.view = Some(view);

        Ok(())
    }

    pub fn detach_view(&mut self) -> Result<(), Error> {
        match self.view.take() {
            Some(mut view) => {
                let _ = view.detach();
                view.release();
            },
            None => {}
        };

        Ok(())
    }

}

fn instance_from_wndproc(hwnd: HWND, message: u32, lparam: LPARAM) -> *mut Window {
    if message == WM_NCCREATE {
        unsafe {
            let create_struct = lparam as *mut CREATESTRUCTA;
            let user_data_ptr = (*create_struct).lpCreateParams;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, user_data_ptr as isize);
            let instance = user_data_ptr as *mut Window;
            instance
        }
    } else {
        unsafe {
            let user_data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut c_void;
            let instance = user_data_ptr as *mut Window;
            instance
        }
    }
}

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let user_data_ptr = instance_from_wndproc(hwnd, message, lparam);
    if user_data_ptr.is_null() {
        unsafe { DefWindowProcA(hwnd, message, wparam, lparam) }
    } else {
        return unsafe { (*user_data_ptr).on_event(hwnd, message, wparam, lparam) };
    }
}

#[inline]
pub fn lo_word(l: u32) -> u16 {
    (l & 0xffff) as u16
}

#[inline]
pub fn hi_word(l: u32) -> u16 {
    ((l >> 16) & 0xffff) as u16
}

#[inline]
pub fn lo_byte(l: u16) -> u8 {
    (l & 0xff) as u8
}

#[inline]
pub fn hi_byte(l: u16) -> u8 {
    ((l >> 8) & 0xff) as u8
}

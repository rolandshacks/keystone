use log::{*};
use vst3_sys::base::IUnknown;
use std::{fs, time::UNIX_EPOCH};

use vst3_com::{ComInterface, VstPtr};


pub fn get_file_time(filename: &str) -> i64 {
    match fs::metadata(filename) {
        Ok(m) => {
            match m.modified() {
                Ok(t) => {
                    match t.duration_since(UNIX_EPOCH) {
                        Ok(d) => (d.as_secs() / 60) as i64, // store minutes since unix epoch
                        Err(_) => 0
                    }
                }
                Err(_) => 0
            }
        }
        Err(_) => 0
    }
}

pub fn slashify_path(path: &str) -> String {
    let mut s = String::new();

    for c in path.chars() {
        if c == '\\' {
            s.push('/');
        } else {
            s.push(c);
        }
    }

    s
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Size {
    pub width: i32,
    pub height: i32
}

impl Size {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height
        }
    }

    pub fn zero(&mut self) {
        self.set(0, 0);
    }

    pub fn set(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    pub fn add(&mut self, width: i32, height: i32) {
        self.width += width;
        self.height += height;
    }

    pub fn add_size(&mut self, size: &Size) {
        self.width += size.width;
        self.height += size.height;
    }

    pub fn diff(a: &Size, b: &Size) -> Size {
        Size::new(
            a.width - b.width,
            a.height - b.height
        )
    }

    pub fn equals(&self, sz: &Size) -> bool {
        return self.width == sz.width && self.height == sz.height
    }

}


#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32
}

impl Rect {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom
        }
    }

    pub fn zero(&mut self) {
        self.set(0, 0, 0, 0);
    }

    pub fn set(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        self.left = left;
        self.top = top;
        self.right =  right;
        self.bottom = bottom;
    }

    pub fn size(&self) -> Size {
        Size::new(self.right - self.left, self.bottom - self.top)
    }

    pub fn equals(&self, rect: &Rect) -> bool {
        return self.left == rect.left && self.top == rect.top && self.right == rect.right && self.bottom == rect.bottom
    }


}

pub fn trace_ref<T: ComInterface + ?Sized>(ptr: &VstPtr<T>) {
    unsafe {
        ptr.add_ref();
        let ref_count = ptr.release();
        trace!("[[ref count: {}]]", ref_count);
    }
}

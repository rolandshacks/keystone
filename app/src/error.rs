//!
//! Error
//!

use std::convert::From;

#[derive(Debug)]
pub struct Error {
    message: String
}

impl From<&Error> for Error {
    #[inline]
    fn from(e: &Error) -> Self {
        Self { message: e.message.clone() }
    }
}

impl From<String> for Error {
    #[inline]
    fn from(s: String) -> Self {
        Self { message: s }
    }
}

impl From<&String> for Error {
    #[inline]
    fn from(s: &String) -> Self {
        Self { message: s.clone() }
    }
}

impl From<&mut String> for Error {
    #[inline]
    fn from(s: &mut String) -> Self {
        Self { message: s.clone() }
    }
}

impl From<&str> for Error {
    #[inline]
    fn from(s: &str) -> Self {
        Self { message: s.to_owned() }
    }
}

impl From<&mut str> for Error {
    #[inline]
    fn from(s: &mut str) -> Self {
        Self { message: s.to_owned() }
    }
}

impl Error {
    pub fn message(&self) -> &String {
        &self.message
    }
}

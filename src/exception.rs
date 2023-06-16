use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Exception {
    RequestIsNotUtf8,
    UnSupportedRequestMethod,
    UnsupportedHttpVersion,
    FileNotFound,
    PHPExecuteFailed,
    PHPCodeError,
}

use Exception::*;

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestIsNotUtf8 => write!(f, "Request bytes can't be parsed in UTF-8"),
            UnSupportedRequestMethod => write!(f, "Unsupported request method"),
            UnsupportedHttpVersion => write!(f, "Unsupported HTTP version"),
            FileNotFound => write!(f, "File not found (404)"),
            PHPExecuteFailed => write!(f, "Couldn't invoke PHP interpreter"),
            PHPCodeError => write!(f, "An error happened in php code"),
        }
    }
}

use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Exception {
    RequestConstructFailed,
}

use Exception::*;

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestConstructFailed => write!(f, "Failed to construct request object from bytes"),
        }
    }
}

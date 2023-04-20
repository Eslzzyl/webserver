use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Exception {
    BindFailed(u16),
}

use Exception::*;

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindFailed(port) => write!(f, "Failed to bind port {:#x}", port),
        }
    }
}

extern crate busmust_sys as ffi;

use std::fmt;
use dmgr::desc_from_error;

mod call;
mod util;
pub mod dmgr;

#[derive(Debug, Clone)]
pub struct Error(ffi::BMStatus);

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {:?} ({})", self, desc_from_error(self))
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

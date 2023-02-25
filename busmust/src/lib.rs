extern crate busmust_sys as ffi;

#[macro_use]
extern crate failure;

mod call;
mod util;

pub mod dmgr;

use std::fmt;

#[derive(Debug, Fail)]
pub struct Error(ffi::BMStatus);

pub type Result<T> = std::result::Result<T, failure::Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = dmgr::desc_from_error(self);
        write!(
            f,
            "An error occurred with error code {}/({})",
            self, desc
        )
    }
}

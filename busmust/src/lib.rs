extern crate busmust_sys as ffi;
extern crate anyhow;
extern crate thiserror;

use thiserror::Error;

mod call;
mod util;
pub mod dmgr;

#[derive(Error, Debug)]
pub enum Error {
    #[error("API error: {0}")]
    BusmustError(ffi::BMStatus)
}

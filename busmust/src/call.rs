use super::{Result, Error};
use ffi::BMStatus;

pub fn cvt_r(r: BMStatus) -> Result<()> {
    if matches!(r, BMStatus::Ok) {
        Ok(())
    } else {
        Err(Error(r).into())
    }
}

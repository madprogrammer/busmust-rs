use super::Error;
use ffi::BMStatus;
use anyhow::Result;

pub fn cvt_r(r: BMStatus) -> Result<()> {
     if matches!(r, BMStatus::Ok) {
        Ok(())
    } else {
        Err(Error::BusmustError(r).into())
    }
}

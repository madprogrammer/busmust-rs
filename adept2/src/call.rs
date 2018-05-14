use super::{dmgr, Result};
use ffi;

pub fn cvt_r(r: ffi::BOOL) -> Result<()> {
    if r != 0 {
        Ok(())
    } else {
        Err(dmgr::get_last_error().into())
    }
}

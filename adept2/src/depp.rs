use super::{DeviceCaps, Error, Result};
use call::cvt_r;
use dmgr::{Device, DeviceHandle};
use ffi;
use std::{cmp, mem};
use std::os::raw::c_char;
use std::time::Duration;
use util::StringExt;

#[repr(C)]
pub struct AddrData(u8, u8);

pub fn get_version() -> Result<String> {
    let version = unsafe {
        let mut version: [c_char; ffi::cchVersionMax] = mem::zeroed();
        cvt_r(ffi::DeppGetVersion(version.as_mut_ptr()))?;
        String::from_slice(&version[..])
    };
    Ok(version)
}

pub struct Depp(DeviceHandle);

impl Depp {
    fn _put_reg(&mut self, addr: u8, data: u8, overlap: bool) -> Result<()> {
        unsafe {
            cvt_r(ffi::DeppPutReg(
                self.0.raw(),
                addr,
                data,
                overlap as ffi::BOOL,
            ))
        }
    }

    fn _get_reg(&mut self, addr: u8, overlap: bool) -> Result<u8> {
        let mut data: u8 = 0;
        unsafe {
            cvt_r(ffi::DeppGetReg(
                self.0.raw(),
                addr,
                &mut data,
                overlap as ffi::BOOL,
            ))?
        }
        Ok(data)
    }

    fn _put_reg_set(&mut self, addr_data: &[AddrData], overlap: bool) -> Result<()> {
        let len = addr_data.len() as u32;
        let addr_data: *mut u8 = addr_data.as_ptr() as *mut u8;
        unsafe {
            cvt_r(ffi::DeppPutRegSet(
                self.0.raw(),
                addr_data,
                len,
                overlap as ffi::BOOL,
            ))
        }
    }

    fn _get_reg_set(&mut self, addr: &[u8], data: &mut [u8], overlap: bool) -> Result<()> {
        let len = cmp::min(addr.len(), data.len()) as u32;
        let addr: *mut u8 = addr.as_ptr() as *mut u8;
        let data: *mut u8 = data.as_mut_ptr();
        unsafe {
            cvt_r(ffi::DeppGetRegSet(
                self.0.raw(),
                addr,
                data,
                len,
                overlap as ffi::BOOL,
            ))
        }
    }

    fn _put_reg_repeat(&mut self, addr: u8, data: &[u8], overlap: bool) -> Result<()> {
        let len = data.len() as u32;
        let data: *mut u8 = data.as_ptr() as *mut u8;
        unsafe {
            cvt_r(ffi::DeppPutRegRepeat(
                self.0.raw(),
                addr,
                data,
                len,
                overlap as ffi::BOOL,
            ))
        }
    }

    fn _get_reg_repeat(&mut self, addr: u8, data: &mut [u8], overlap: bool) -> Result<()> {
        let len = data.len() as u32;
        let data: *mut u8 = data.as_mut_ptr();
        let overlap = overlap as ffi::BOOL;
        unsafe {
            cvt_r(ffi::DeppGetRegRepeat(
                self.0.raw(),
                addr,
                data,
                len,
                overlap,
            ))
        }
    }

    pub fn new(device: &Device) -> Result<Depp> {
        device
            .device_caps()
            .and_then(|caps| {
                if caps.contains(DeviceCaps::EPP) {
                    device.open()
                } else {
                    Err(Error(ffi::ercNotSupported).into())
                }
            })
            .and_then(|hif| {
                enable(&hif)?;
                Ok(Depp(hif))
            })
    }

    /* Basic interface functions. */
    pub fn port_count(&self) -> Result<usize> {
        let mut count: i32 = 0;
        unsafe {
            cvt_r(ffi::DeppGetPortCount(self.0.raw(), &mut count))?;
        }
        Ok(count as usize)
    }

    pub fn port_properties(&self, port: i32) -> Result<u32> {
        let mut props: u32 = 0;
        unsafe {
            cvt_r(ffi::DeppGetPortProperties(self.0.raw(), port, &mut props))?;
        }
        Ok(props)
    }

    pub fn enable_port(&mut self, port: i32) -> Result<()> {
        unsafe { cvt_r(ffi::DeppEnableEx(self.0.raw(), port)) }
    }

    /* Data transfer functions */
    pub fn put_reg(&mut self, addr: u8, data: u8) -> Result<()> {
        self._put_reg(addr, data, false)
    }

    pub fn get_reg(&mut self, addr: u8) -> Result<u8> {
        self._get_reg(addr, false)
    }

    pub fn put_reg_set(&mut self, addr_data: &[AddrData]) -> Result<()> {
        self._put_reg_set(addr_data, false)
    }

    pub fn get_reg_set(&mut self, addr: &[u8], data: &mut [u8]) -> Result<()> {
        self._get_reg_set(addr, data, false)
    }

    pub fn put_reg_repeat(&mut self, addr: u8, data: &[u8]) -> Result<()> {
        self._put_reg_repeat(addr, data, false)
    }

    pub fn get_reg_repeat(&mut self, addr: u8, data: &mut [u8]) -> Result<()> {
        self._get_reg_repeat(addr, data, false)
    }

    pub fn put_reg_with_overlap(&mut self, addr: u8, data: u8) -> Result<()> {
        self._put_reg(addr, data, true)
    }

    pub fn get_reg_with_overlap(&mut self, addr: u8) -> Result<u8> {
        self._get_reg(addr, true)
    }

    pub fn put_reg_set_with_overlap(&mut self, addr_data: &[AddrData]) -> Result<()> {
        self._put_reg_set(addr_data, true)
    }

    pub fn get_reg_set_with_overlap(&mut self, addr: &[u8], data: &mut [u8]) -> Result<()> {
        self._get_reg_set(addr, data, true)
    }

    pub fn put_reg_repeat_with_overlap(&mut self, addr: u8, data: &[u8]) -> Result<()> {
        self._put_reg_repeat(addr, data, true)
    }

    pub fn get_reg_repeat_with_overlap(&mut self, addr: u8, data: &mut [u8]) -> Result<()> {
        self._get_reg_repeat(addr, data, true)
    }

    /* Misc. control functions */
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<Duration> {
        let timeout = as_nanos(&timeout);
        assert!(timeout <= as_nanos(&Duration::new(0, u32::max_value() as _)));
        let timeout = timeout as u32;
        let mut actual: u32 = 0;
        unsafe {
            cvt_r(ffi::DeppSetTimeout(self.0.raw(), timeout, &mut actual))?;
        }
        Ok(Duration::new(0, actual as _))
    }
}

impl Drop for Depp {
    fn drop(&mut self) {
        let _ = disable(&self.0);
    }
}

pub fn enable(hif: &DeviceHandle) -> Result<()> {
    unsafe { cvt_r(ffi::DeppEnable(hif.raw())) }
}

pub fn disable(hif: &DeviceHandle) -> Result<()> {
    unsafe { cvt_r(ffi::DeppDisable(hif.raw())) }
}

fn as_nanos(dur: &Duration) -> u64 {
    dur.as_secs() * 1_000_000_000 + dur.subsec_nanos() as u64
}

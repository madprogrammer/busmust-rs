use super::{DeviceCaps, Error, ProductId, Result};
use call::cvt_r;
use std::mem;
use std::os::raw::{c_char, c_int};

use ffi;
use util::StringExt;

pub fn get_version() -> Result<String> {
    let version = unsafe {
        let mut version: [c_char; ffi::cchVersionMax] = mem::zeroed();
        cvt_r(ffi::DmgrGetVersion(version.as_mut_ptr()))?;
        String::from_slice(&version[..])
    };
    Ok(version)
}

pub fn get_last_error() -> Error {
    let err = unsafe { ffi::DmgrGetLastError() };
    Error(err)
}

pub fn desc_from_error(err: &Error) -> (String, String) {
    unsafe {
        let mut symbolic: [c_char; ffi::cchErcMax] = mem::zeroed();
        let mut desc: [c_char; ffi::cchErcMsgMax] = mem::zeroed();

        let r = ffi::DmgrSzFromErc(err.0, symbolic[..].as_mut_ptr(), desc[..].as_mut_ptr());
        assert!(r == ffi::ercNoErc);

        let symbolic = String::from_slice(&symbolic[..]);
        let desc = String::from_slice(&desc[..]);

        (symbolic, desc)
    }
}

#[derive(Debug)]
pub struct DeviceHandle {
    hif: ffi::HIF,
}

impl DeviceHandle {
    pub fn new(hif: ffi::HIF) -> DeviceHandle {
        DeviceHandle { hif }
    }

    pub fn raw(&self) -> ffi::HIF {
        self.hif
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        let _ = unsafe { ffi::DmgrClose(self.hif) };
    }
}

pub struct Device(ffi::DVC);

impl Device {
    pub fn open(&self) -> Result<DeviceHandle> {
        let mut hif: ffi::HIF = 0;
        unsafe {
            cvt_r(ffi::DmgrOpen(&mut hif, self.0.szConn.as_ptr() as *mut _))?;
        }
        Ok(DeviceHandle::new(hif))
    }

    pub fn name(&self) -> String {
        String::from_slice(&self.0.szName[..])
    }

    pub fn conn(&self) -> String {
        String::from_slice(&self.0.szConn[..])
    }

    pub fn alias(&self) -> Result<String> {
        unsafe {
            let mut alias: [c_char; ffi::cchAliasMax] = mem::zeroed();
            cvt_r(ffi::DmgrGetInfo(
                &self.0 as *const _ as *mut _,
                ffi::dinfoAlias,
                alias.as_mut_ptr() as *mut _,
            ))?;
            Ok(String::from_slice(&alias[..]))
        }
    }

    pub fn user_name(&self) -> Result<String> {
        unsafe {
            let mut username: [c_char; ffi::cchUsrNameMax] = mem::zeroed();
            cvt_r(ffi::DmgrGetInfo(
                &self.0 as *const _ as *mut _,
                ffi::dinfoUsrName,
                username.as_mut_ptr() as *mut _,
            ))?;
            Ok(String::from_slice(&username[..]))
        }
    }

    pub fn product_name(&self) -> Result<String> {
        unsafe {
            let mut prodname: [c_char; ffi::cchProdNameMax] = mem::zeroed();
            cvt_r(ffi::DmgrGetInfo(
                &self.0 as *const _ as *mut _,
                ffi::dinfoProdName,
                prodname.as_mut_ptr() as *mut _,
            ))?;
            Ok(String::from_slice(&prodname[..]))
        }
    }

    pub fn product_id(&self) -> Result<ProductId> {
        let mut pdid: ffi::PDID = 0;
        unsafe {
            cvt_r(ffi::DmgrGetInfo(
                &self.0 as *const _ as *mut _,
                ffi::dinfoPDID,
                &mut pdid as *mut _ as *mut _,
            ))?;
        }
        Ok(ProductId(pdid))
    }

    pub fn serial_number(&self) -> Result<String> {
        unsafe {
            let mut sn: [c_char; ffi::cchSnMax] = mem::zeroed();
            cvt_r(ffi::DmgrGetInfo(
                &self.0 as *const _ as *mut _,
                ffi::dinfoSN,
                sn.as_mut_ptr() as *mut _,
            ))?;
            Ok(String::from_slice(&sn[..]))
        }
    }

    //    pub fn ip(&self) -> String {}

    //  pub fn mac(&self) -> String {}

    pub fn device_caps(&self) -> Result<DeviceCaps> {
        let mut dcap: ffi::DCAP = 0;
        unsafe {
            cvt_r(ffi::DmgrGetInfo(
                &self.0 as *const _ as *mut _,
                ffi::dinfoPDID,
                &mut dcap as *mut _ as *mut _,
            ))?;
        }
        Ok(DeviceCaps::from_bits_truncate(dcap))
    }

    //    pub fn SerParam
    //    pub fn ParAddr
    //    pub fn UsbPath

    //    pub fn product_id(&self) -> usize {}

    pub fn open_count(&self) -> Result<usize> {
        let mut count: u32 = 0;
        unsafe {
            cvt_r(ffi::DmgrGetInfo(
                &self.0 as *const _ as *mut _,
                ffi::dinfoOpenCount,
                &mut count as *mut _ as *mut _,
            ))?;
        }
        Ok(count as usize)
    }

    //    pub fn firmware_version(&self) -> usize {}
}

pub struct Devices {
    curr: c_int,
    count: c_int,
}

impl Devices {
    fn new(count: c_int) -> Self {
        Devices { curr: 0, count }
    }
}

impl Iterator for Devices {
    type Item = Device;

    fn next(&mut self) -> Option<Device> {
        if self.curr < self.count {
            let device = unsafe {
                let mut dvc: ffi::DVC = mem::zeroed();
                if ffi::DmgrGetDvc(self.curr, &mut dvc) == 0 {
                    return None;
                }
                Device(dvc)
            };
            self.curr += 1;

            Some(device)
        } else {
            unsafe {
                ffi::DmgrFreeDvcEnum();
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.curr) as usize;
        (remaining, Some(remaining))
    }
}

pub fn enum_devices() -> Result<Devices> {
    let mut count: c_int = 0;
    unsafe { cvt_r(ffi::DmgrEnumDevices(&mut count))? }
    Ok(Devices::new(count))
}
